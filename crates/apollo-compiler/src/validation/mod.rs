//! Supporting APIs for [GraphQL validation](https://spec.graphql.org/October2021/#sec-Validation)
//! and other kinds of errors.

#[cfg(doc)]
use crate::{ExecutableDocument, Schema};

mod argument;
mod directive;
mod enum_;
mod field;
mod fragment;
mod input_object;
mod interface;
mod object;
pub(crate) mod operation;
mod scalar;
mod schema;
pub(crate) mod selection;
mod union_;
mod validation_db;
mod value;
mod variable;

use crate::ast::Name;
use crate::executable::BuildError as ExecutableBuildError;
use crate::execution::{GraphQLError, GraphQLLocation};
use crate::schema::BuildError as SchemaBuildError;
use crate::Node;
use crate::SourceFile;
use crate::SourceMap;
use indexmap::IndexSet;
use std::fmt;
use std::io;
use std::sync::Arc;
use std::sync::OnceLock;
pub(crate) use validation_db::{ValidationDatabase, ValidationStorage};

pub use crate::database::FileId;
pub use crate::node::NodeLocation;

/// Wraps a [`Schema`] or [`ExecutableDocument`] to mark it
/// as [valid](https://spec.graphql.org/October2021/#sec-Validation).
///
/// This is obtained either by running validation with one of:
///
/// * [`Schema::parse_and_validate`]
/// * [`Schema::validate`]
/// * [`ExecutableDocument::parse_and_validate`]
/// * [`ExecutableDocument::validate`]
///
/// … or by explicitly skipping it with [`Valid::assume_valid`].
///
/// The schema or document inside `Valid<T>` is immutable (`&mut T` is not given out).
/// It can be extracted with [`into_inner`][Self::into_inner],
/// such as to mutate it then possibly re-validate it.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Valid<T>(pub(crate) T);

impl<T> Valid<T> {
    /// Construct a `Valid` document without actually running validation.
    ///
    /// The caller takes responsibility to ascertain that
    /// the document is known through some other means to be valid.
    /// For example, if it was loaded from some external storag
    /// where it was only stored after validation.
    pub fn assume_valid(document: T) -> Self {
        Self(document)
    }

    /// Extract the schema or document, such as to mutate it then possibly re-validate it.
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> std::ops::Deref for Valid<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> AsRef<T> for Valid<T> {
    fn as_ref(&self) -> &T {
        &self.0
    }
}

impl<T: fmt::Display> fmt::Display for Valid<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// A conversion failed with some errors, but also resulted in a partial document.
///
/// The [`Debug`][fmt::Debug] trait is implemented by forwarding to [`Self::errors`] and
/// ignoring [`Self::partial`].
/// This is so that the panic message prints (only) errors when [`.unwrap()`][Result::unwrap]
/// is called on a `Result<_, WithError<_>>` value as returned by various APIs.
pub struct WithErrors<T> {
    /// The partial result of the conversion.
    /// Some components may be missing,
    /// for example if an error causes them not to be representable in the target data structure.
    pub partial: T,

    /// Errors collected during the conversion.
    /// Should be non-empty when `WithError` is returned.
    pub errors: DiagnosticList,
}

impl<T> fmt::Debug for WithErrors<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.errors.fmt(f)
    }
}

impl<T> fmt::Display for WithErrors<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.errors.fmt(f)
    }
}

/// A collection of diagnostics returned by some validation method
#[derive(Clone)]
pub struct DiagnosticList {
    pub(crate) sources: SourceMap,
    diagnostics_data: Vec<DiagnosticData>,
}

#[derive(Clone)]
pub(crate) struct DiagnosticData {
    location: Option<NodeLocation>,
    details: Details,
}

/// A single diagnostic in a [`DiagnosticList`]
pub struct Diagnostic<'a> {
    sources: &'a SourceMap,
    data: &'a DiagnosticData,
}

#[derive(thiserror::Error, Debug, Clone)]
pub(crate) enum Details {
    #[error("{message}")]
    ParserLimit { message: String },
    #[error("syntax error: {message}")]
    SyntaxError { message: String },
    #[error("{0}")]
    SchemaBuildError(SchemaBuildError),
    #[error("{0}")]
    ExecutableBuildError(ExecutableBuildError),
    #[error("compiler error: {0}")]
    CompilerDiagnostic(crate::ApolloDiagnostic),
}

impl DiagnosticData {
    fn report(&self, color: bool) -> ariadne::Report<'static, NodeLocation> {
        let config = ariadne::Config::default().with_color(color);
        if let Details::CompilerDiagnostic(diagnostic) = &self.details {
            return diagnostic.to_report(config);
        }

        let (id, offset) = if let Some(location) = self.location {
            (location.file_id(), location.offset())
        } else {
            (FileId::NONE, 0)
        };

        let mut report = ariadne::Report::build::<FileId>(ariadne::ReportKind::Error, id, offset)
            .with_config(config);
        let mut colors = ariadne::ColorGenerator::new();
        macro_rules! opt_label {
            ($location: expr, $message: literal $(, $args: expr )* $(,)?) => {
                if let Some(location) = $location {
                    report.add_label(
                        ariadne::Label::new(*location)
                            .with_message(format_args!($message $(, $args)*))
                            .with_color(colors.next()),
                    )
                }
            };
            ($message: literal $(, $args: expr )* $(,)?) => {
                opt_label!(&self.location, $message $(, $args)*)
            };
        }
        // Main message from `derive(thiserror::Error)` based on `#[error("…")]` attributes:
        report.set_message(&self.details);
        // Every case should also have a label at the main location
        // (preferably saying something not completely redundant with the main message)
        // and may have additional labels.
        // Labels are always optional because locations are always optional,
        // so essential information should be in the main message.
        match &self.details {
            Details::CompilerDiagnostic(_) => unreachable!(),
            Details::ParserLimit { message, .. } => opt_label!("{message}"),
            Details::SyntaxError { message, .. } => opt_label!("{message}"),
            Details::SchemaBuildError(err) => match err {
                SchemaBuildError::ExecutableDefinition { .. } => {
                    opt_label!("remove this definition, or use `parse_mixed()`")
                }
                SchemaBuildError::SchemaDefinitionCollision {
                    previous_location, ..
                } => {
                    opt_label!(previous_location, "previous `schema` definition here");
                    opt_label!("`schema` redefined here");
                    report.set_help(
                        "merge this definition with the previous one, or use `extend schema`",
                    );
                }
                SchemaBuildError::DirectiveDefinitionCollision {
                    previous_location,
                    name,
                    ..
                } => {
                    opt_label!(previous_location, "previous definition of `@{name}` here");
                    opt_label!("`@{name}` redefined here");
                    report.set_help("remove or rename one of the definitions");
                }
                SchemaBuildError::TypeDefinitionCollision {
                    previous_location,
                    name,
                    ..
                } => {
                    opt_label!(previous_location, "previous definition of `{name}` here");
                    opt_label!("`{name}` redefined here");
                    report.set_help("remove or rename one of the definitions, or use `extend`");
                }
                SchemaBuildError::BuiltInScalarTypeRedefinition { .. } => {
                    opt_label!("remove this scalar definition");
                }
                SchemaBuildError::OrphanSchemaExtension { .. } => opt_label!("extension here"),
                SchemaBuildError::OrphanTypeExtension { .. } => opt_label!("extension here"),
                SchemaBuildError::TypeExtensionKindMismatch { def_location, .. } => {
                    opt_label!(def_location, "type definition");
                    opt_label!("extension here")
                }
                SchemaBuildError::DuplicateRootOperation {
                    previous_location,
                    operation_type,
                    ..
                } => {
                    opt_label!(
                        previous_location,
                        "previous definition of `{operation_type}` here"
                    );
                    opt_label!("`{operation_type}` redefined here");
                }
                SchemaBuildError::DuplicateImplementsInterfaceInObject {
                    name_at_previous_location,
                    ..
                }
                | SchemaBuildError::DuplicateImplementsInterfaceInInterface {
                    name_at_previous_location,
                    ..
                } => {
                    let previous_location = &name_at_previous_location.location();
                    let name = name_at_previous_location;
                    opt_label!(
                        previous_location,
                        "previous implementation of `{name}` here"
                    );
                    opt_label!("`{name}` implemented again here");
                }
                SchemaBuildError::ObjectFieldNameCollision {
                    name_at_previous_location,
                    ..
                }
                | SchemaBuildError::InterfaceFieldNameCollision {
                    name_at_previous_location,
                    ..
                }
                | SchemaBuildError::EnumValueNameCollision {
                    name_at_previous_location,
                    ..
                }
                | SchemaBuildError::UnionMemberNameCollision {
                    name_at_previous_location,
                    ..
                }
                | SchemaBuildError::InputFieldNameCollision {
                    name_at_previous_location,
                    ..
                } => {
                    let previous_location = &name_at_previous_location.location();
                    let name = name_at_previous_location;
                    opt_label!(previous_location, "previous definition of `{name}` here");
                    opt_label!("`{name}` redefined here");
                }
            },
            Details::ExecutableBuildError(err) => match err {
                ExecutableBuildError::TypeSystemDefinition { .. } => {
                    opt_label!("remove this definition, or use `parse_mixed()`")
                }
                ExecutableBuildError::AmbiguousAnonymousOperation { .. } => {
                    opt_label!("provide a name for this definition");
                    report.set_help(
                        "GraphQL requires operations to be named if the document has more than one",
                    );
                }
                ExecutableBuildError::OperationNameCollision {
                    name_at_previous_location,
                    ..
                }
                | ExecutableBuildError::FragmentNameCollision {
                    name_at_previous_location,
                    ..
                } => {
                    let previous_location = &name_at_previous_location.location();
                    let name = name_at_previous_location;
                    opt_label!(previous_location, "previous definition of `{name}` here");
                    opt_label!("`{name}` redefined here");
                }
                ExecutableBuildError::UndefinedRootOperation { operation_type, .. } => {
                    opt_label!(
                        "`{operation_type}` is not defined in the schema \
                         and is therefore not supported"
                    );
                    report.set_help(format_args!(
                        "consider defining a `{operation_type}` root operation type \
                         in your schema"
                    ))
                }
                ExecutableBuildError::UndefinedTypeInNamedFragmentTypeCondition { .. } => {
                    opt_label!("type condition here")
                }
                ExecutableBuildError::UndefinedTypeInInlineFragmentTypeCondition {
                    path, ..
                } => {
                    opt_label!("type condition here");
                    report.set_note(format_args!("path to the inline fragment: `{path} → ...`"))
                }
                ExecutableBuildError::SubselectionOnScalarType { path, .. }
                | ExecutableBuildError::SubselectionOnEnumType { path, .. } => {
                    opt_label!("remove subselections here");
                    report.set_note(format_args!("path to the field: `{path}`"))
                }
                ExecutableBuildError::UndefinedField {
                    field_name,
                    type_name,
                    path,
                    ..
                } => {
                    opt_label!("field `{field_name}` selected here");
                    opt_label!(&type_name.location(), "type `{type_name}` defined here");
                    report.set_note(format_args!("path to the field: `{path}`"))
                }
            },
        }
        report.finish()
    }
}

impl<'a> Diagnostic<'a> {
    /// Get the line and column number where this diagnostic was raised.
    pub fn get_line_column(&self) -> Option<GraphQLLocation> {
        let loc = self.data.location?;
        let source = self.sources.get(&loc.file_id)?;
        source
            .get_line_column(loc.offset())
            .map(|(line, column)| GraphQLLocation {
                line: line + 1,
                column: column + 1,
            })
    }

    /// Get serde_json serialisable version of the current diagnostic.
    pub fn to_json(&self) -> GraphQLError {
        let locations = self.get_line_column().into_iter().collect();

        GraphQLError {
            message: self.message().to_string(),
            locations,
        }
    }

    pub fn message(&self) -> &impl fmt::Display {
        &self.data.details
    }
}

impl DiagnosticList {
    /// Creates an empty diagnostic list with the given source map.
    pub fn new(sources: SourceMap) -> Self {
        Self {
            sources,
            diagnostics_data: Vec::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.diagnostics_data.is_empty()
    }

    pub fn len(&self) -> usize {
        self.diagnostics_data.len()
    }

    pub fn iter(
        &self,
    ) -> impl Iterator<Item = Diagnostic<'_>> + DoubleEndedIterator + ExactSizeIterator {
        self.diagnostics_data.iter().map(|data| Diagnostic {
            sources: &self.sources,
            data,
        })
    }

    /// Returns a human-readable string formatting, without color codes regardless of stderr.
    ///
    /// `Display` and `.to_string()` are meant for printing to stderr,
    /// and will include ANSI color codes if stderr is detected to be a terminal.
    pub fn to_string_no_color(&self) -> String {
        format!("{self:#}")
    }

    pub(crate) fn push(&mut self, location: Option<NodeLocation>, details: impl Into<Details>) {
        self.diagnostics_data.push(DiagnosticData {
            location,
            details: details.into(),
        })
    }

    /// Concatenate an `other` list of diagnostics into `self`, and sort them together.
    pub fn merge(&mut self, other: Self) {
        if !Arc::ptr_eq(&self.sources, &other.sources) {
            let sources = Arc::make_mut(&mut self.sources);
            for (&k, v) in &*other.sources {
                sources.entry(k).or_insert_with(|| v.clone());
            }
        }
        self.diagnostics_data.extend(other.diagnostics_data);
        self.sort()
    }

    fn sort(&mut self) {
        self.diagnostics_data
            .sort_by_key(|err| err.location.map(|loc| (loc.file_id(), loc.offset())));
    }

    pub(crate) fn into_result(mut self) -> Result<(), Self> {
        if self.diagnostics_data.is_empty() {
            Ok(())
        } else {
            self.sort();
            Err(self)
        }
    }

    pub(crate) fn into_result_with<T>(self, value: T) -> Result<T, WithErrors<T>> {
        match self.into_result() {
            Ok(()) => Ok(value),
            Err(errors) => Err(WithErrors {
                partial: value,
                errors,
            }),
        }
    }

    pub(crate) fn into_valid_result<T>(self, value: T) -> Result<Valid<T>, WithErrors<T>> {
        match self.into_result() {
            Ok(()) => Ok(Valid(value)),
            Err(errors) => Err(WithErrors {
                partial: value,
                errors,
            }),
        }
    }
}

/// Defaults to ANSI color codes if stderr is a terminal.
///
/// Use alternate formatting to never use colors: `format!("{diagnostics:#}")`
impl fmt::Display for DiagnosticList {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for diagnostic in self.iter() {
            diagnostic.fmt(f)?
        }
        Ok(())
    }
}

/// Defaults to ANSI color codes if stderr is a terminal.
///
/// Use alternate formatting to never use colors: `format!("{diagnostic:#}")`
impl fmt::Display for Diagnostic<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        struct Adaptor<'a, 'b>(&'a mut fmt::Formatter<'b>);

        impl io::Write for Adaptor<'_, '_> {
            fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
                let s = std::str::from_utf8(buf).map_err(|_| io::ErrorKind::Other)?;
                self.0.write_str(s).map_err(|_| io::ErrorKind::Other)?;
                Ok(buf.len())
            }

            fn flush(&mut self) -> io::Result<()> {
                Ok(())
            }
        }

        let color = !f.alternate();
        self.data
            .report(color)
            .write(Cache(self.sources), Adaptor(f))
            .map_err(|_| fmt::Error)
    }
}

struct Cache<'a>(&'a SourceMap);

impl ariadne::Cache<FileId> for Cache<'_> {
    fn fetch(&mut self, file_id: &FileId) -> Result<&ariadne::Source, Box<dyn fmt::Debug + '_>> {
        struct NotFound(FileId);
        impl fmt::Debug for NotFound {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "source file not found: {:?}", self.0)
            }
        }
        if let Some(source_file) = self.0.get(file_id) {
            Ok(source_file.ariadne())
        } else if *file_id == FileId::NONE || *file_id == FileId::HACK_TMP {
            static EMPTY: OnceLock<ariadne::Source> = OnceLock::new();
            Ok(EMPTY.get_or_init(|| ariadne::Source::from("")))
        } else {
            Err(Box::new(NotFound(*file_id)))
        }
    }

    fn display<'a>(&self, file_id: &'a FileId) -> Option<Box<dyn fmt::Display + 'a>> {
        if *file_id != FileId::NONE {
            struct Path(Arc<SourceFile>);
            impl fmt::Display for Path {
                fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                    self.0.path().display().fmt(f)
                }
            }
            let source_file = self.0.get(file_id)?;
            Some(Box::new(Path(source_file.clone())))
        } else {
            struct NoSourceFile;
            impl fmt::Display for NoSourceFile {
                fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                    f.write_str("(no source file)")
                }
            }
            Some(Box::new(NoSourceFile))
        }
    }
}

impl fmt::Debug for DiagnosticList {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

impl ariadne::Span for NodeLocation {
    type SourceId = FileId;

    fn source(&self) -> &FileId {
        &self.file_id
    }

    fn start(&self) -> usize {
        self.offset()
    }

    fn end(&self) -> usize {
        self.end_offset()
    }
}

impl From<SchemaBuildError> for Details {
    fn from(value: SchemaBuildError) -> Self {
        Details::SchemaBuildError(value)
    }
}

impl From<ExecutableBuildError> for Details {
    fn from(value: ExecutableBuildError) -> Self {
        Details::ExecutableBuildError(value)
    }
}

const DEFAULT_RECURSION_LIMIT: usize = 32;

#[derive(Debug, Clone, thiserror::Error)]
#[error("Recursion limit reached")]
#[non_exhaustive]
struct RecursionLimitError {}

/// Track used names in a recursive function.
#[derive(Debug)]
struct RecursionStack {
    seen: IndexSet<Name>,
    high: usize,
    limit: usize,
}

impl RecursionStack {
    fn new() -> Self {
        Self {
            seen: IndexSet::new(),
            high: 0,
            limit: DEFAULT_RECURSION_LIMIT,
        }
    }

    fn with_root(root: Name) -> Self {
        let mut stack = Self::new();
        stack.seen.insert(root);
        stack
    }

    fn with_limit(mut self, limit: usize) -> Self {
        self.limit = limit;
        self
    }

    /// Return the actual API for tracking recursive uses.
    pub(crate) fn guard(&mut self) -> RecursionGuard<'_> {
        RecursionGuard(self)
    }
}

/// Track used names in a recursive function.
///
/// Pass the result of `guard.push(name)` to recursive calls. Use `guard.contains(name)` to check
/// if the name was used somewhere up the call stack. When a guard is dropped, its name is removed
/// from the list.
struct RecursionGuard<'a>(&'a mut RecursionStack);
impl RecursionGuard<'_> {
    /// Mark that we saw a name. If there are too many names, return an error.
    fn push(&mut self, name: &Name) -> Result<RecursionGuard<'_>, RecursionLimitError> {
        debug_assert!(
            self.0.seen.insert(name.clone()),
            "cannot push the same name twice to RecursionGuard, check contains() first"
        );
        self.0.high = self.0.high.max(self.0.seen.len());
        if self.0.seen.len() > self.0.limit {
            Err(RecursionLimitError {})
        } else {
            Ok(RecursionGuard(self.0))
        }
    }
    /// Check if we saw a name somewhere up the call stack.
    fn contains(&self, name: &Name) -> bool {
        self.0.seen.iter().any(|seen| seen == name)
    }
    /// Return the name where we started.
    fn first(&self) -> Option<&Name> {
        self.0.seen.first()
    }
}

impl Drop for RecursionGuard<'_> {
    fn drop(&mut self) {
        // This may already be empty if it's the original `stack.guard()` result, but that's fine
        let _ = self.0.seen.pop();
    }
}

/// Errors that can happen when chasing potentially cyclical references.
#[derive(Debug, Clone, thiserror::Error)]
enum CycleError<T> {
    /// Detected a cycle, value contains the path from the offending node back to the node where we
    /// started.
    #[error("Cycle detected")]
    Recursed(Vec<Node<T>>),
    /// Ran into recursion limit before a cycle could be detected.
    #[error(transparent)]
    Limit(#[from] RecursionLimitError),
}

impl<T> CycleError<T> {
    fn trace(mut self, node: &Node<T>) -> Self {
        if let Self::Recursed(trace) = &mut self {
            trace.push(node.clone());
        }
        self
    }
}
