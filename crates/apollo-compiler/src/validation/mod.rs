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
use crate::diagnostic::{Diagnostic, DiagnosticReport, ToDiagnostic};
use crate::executable::BuildError as ExecutableBuildError;
use crate::execution::{GraphQLError, GraphQLLocation, Response};
use crate::schema::BuildError as SchemaBuildError;
use crate::Node;
use crate::SourceMap;
use indexmap::IndexSet;
use std::fmt;
use std::sync::Arc;
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
/// * [`coerce_variable_values`][crate::execution::coerce_variable_values]
///
/// … or by explicitly skipping it with [`Valid::assume_valid`].
///
/// The schema or document inside `Valid<T>` is immutable (`&mut T` is not given out).
/// It can be extracted with [`into_inner`][Self::into_inner],
/// such as to mutate it then possibly re-validate it.
#[derive(Debug, Clone, Eq, PartialEq)]
#[repr(transparent)]
pub struct Valid<T>(pub(crate) T);

impl<T> Valid<T> {
    /// Construct a `Valid` document without actually running validation.
    ///
    /// This takes ownership of the document.
    /// See also [`assume_valid_ref`][Self::assume_valid_ref] which only requires a reference.
    ///
    /// The caller takes responsibility to ascertain that
    /// the document is known through some other means to be valid.
    /// For example, if it was loaded from some external storage
    /// where it was only stored after validation.
    pub fn assume_valid(document: T) -> Self {
        Self(document)
    }

    /// Mark a reference as `Valid` without actually running validation.
    ///
    /// See also [`assume_valid`][Self::assume_valid] returns an owned `Valid<T>`
    /// instead of only a reference.
    ///
    /// The caller takes responsibility to ascertain that
    /// the document is known through some other means to be valid.
    /// For example, if it was loaded from some external storage
    /// where it was only stored after validation.
    pub fn assume_valid_ref(document: &T) -> &Self {
        let ptr: *const T = document;
        let ptr: *const Valid<T> = ptr.cast();
        // SAFETY: `repr(transparent)` makes it valid to transmute `&T` to `&Valid<T>`:
        // <https://doc.rust-lang.org/nomicon/other-reprs.html#reprtransparent>
        unsafe { &*ptr }
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

/// Returned as an error for situtations that should not happen with a valid schema or document.
///
/// Since the relevant APIs take [`Valid<_>`][crate::validation::Valid] parameters,
/// either apollo-compiler has a validation bug
/// or [`assume_valid`][crate::validation::Valid::assume_valid] was used incorrectly.
///
/// Can be [converted][std::convert] to [`GraphQLError`],
/// which populates [`extensions`][GraphQLError::extensions]
/// with a `"APOLLO_SUSPECTED_VALIDATION_BUG": true` entry.
#[derive(Debug, Clone)]
pub struct SuspectedValidationBug {
    pub message: String,
    pub location: Option<NodeLocation>,
}

impl SuspectedValidationBug {
    /// Convert into a JSON-serializable error as represented in a GraphQL response
    pub fn into_graphql_error(self, sources: &SourceMap) -> GraphQLError {
        let Self { message, location } = self;
        let mut err = GraphQLError::new(message, location, sources);
        err.extensions
            .insert("APOLLO_SUSPECTED_VALIDATION_BUG", true.into());
        err
    }

    /// Convert into a response with this error as a [request error]
    /// that prevented execution from starting.
    ///
    /// [request error]: https://spec.graphql.org/October2021/#sec-Errors.Request-errors
    pub fn into_response(self, sources: &SourceMap) -> Response {
        Response::from_request_error(self.into_graphql_error(sources))
    }
}

/// A collection of diagnostics returned by some validation method
#[derive(Clone)]
pub struct DiagnosticList {
    pub(crate) sources: SourceMap,
    diagnostics_data: Vec<DiagnosticData>,
}

/// TODO(@goto-bus-stop): ideally keep this non public
#[derive(Clone)]
pub struct DiagnosticData {
    location: Option<NodeLocation>,
    details: Details,
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

impl ToDiagnostic for DiagnosticData {
    fn location(&self) -> Option<NodeLocation> {
        self.location
    }

    fn report(&self, report: &mut DiagnosticReport) {
        if let Details::CompilerDiagnostic(diagnostic) = &self.details {
            diagnostic.report(report);
            return;
        }

        // Main message from `derive(thiserror::Error)` based on `#[error("…")]` attributes:
        report.with_message(&self.details);

        // Every case should also have a label at the main location
        // (preferably saying something not completely redundant with the main message)
        // and may have additional labels.
        // Labels are always optional because locations are always optional,
        // so essential information should be in the main message.
        match &self.details {
            Details::CompilerDiagnostic(_) => unreachable!(),
            Details::ParserLimit { message, .. } => report.with_label_opt(self.location, message),
            Details::SyntaxError { message, .. } => report.with_label_opt(self.location, message),
            Details::SchemaBuildError(err) => match err {
                SchemaBuildError::ExecutableDefinition { .. } => report.with_label_opt(
                    self.location,
                    "remove this definition, or use `parse_mixed()`",
                ),
                SchemaBuildError::SchemaDefinitionCollision {
                    previous_location, ..
                } => {
                    report.with_label_opt(*previous_location, "previous `schema` definition here");
                    report.with_label_opt(self.location, "`schema` redefined here");
                    report.with_help(
                        "merge this definition with the previous one, or use `extend schema`",
                    );
                }
                SchemaBuildError::DirectiveDefinitionCollision {
                    previous_location,
                    name,
                    ..
                } => {
                    report.with_label_opt(
                        *previous_location,
                        format_args!("previous definition of `@{name}` here"),
                    );
                    report.with_label_opt(self.location, format_args!("`@{name}` redefined here"));
                    report.with_help("remove or rename one of the definitions");
                }
                SchemaBuildError::TypeDefinitionCollision {
                    previous_location,
                    name,
                    ..
                } => {
                    report.with_label_opt(
                        *previous_location,
                        format_args!("previous definition of `{name}` here"),
                    );
                    report.with_label_opt(self.location, format_args!("`{name}` redefined here"));
                    report.with_help("remove or rename one of the definitions, or use `extend`");
                }
                SchemaBuildError::BuiltInScalarTypeRedefinition { .. } => {
                    report.with_label_opt(self.location, "remove this scalar definition");
                }
                SchemaBuildError::OrphanSchemaExtension { .. } => {
                    report.with_label_opt(self.location, "extension here")
                }
                SchemaBuildError::OrphanTypeExtension { .. } => {
                    report.with_label_opt(self.location, "extension here")
                }
                SchemaBuildError::TypeExtensionKindMismatch { def_location, .. } => {
                    report.with_label_opt(*def_location, "type definition");
                    report.with_label_opt(self.location, "extension here")
                }
                SchemaBuildError::DuplicateRootOperation {
                    previous_location,
                    operation_type,
                    ..
                } => {
                    report.with_label_opt(
                        *previous_location,
                        format_args!("previous definition of `{operation_type}` here"),
                    );
                    report.with_label_opt(
                        self.location,
                        format_args!("`{operation_type}` redefined here"),
                    );
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
                    report.with_label_opt(
                        *previous_location,
                        format_args!("previous implementation of `{name}` here"),
                    );
                    report.with_label_opt(
                        self.location,
                        format_args!("`{name}` implemented again here"),
                    );
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
                    report.with_label_opt(
                        *previous_location,
                        format_args!("previous definition of `{name}` here"),
                    );
                    report.with_label_opt(self.location, format_args!("`{name}` redefined here"));
                }
            },
            Details::ExecutableBuildError(err) => match err {
                ExecutableBuildError::TypeSystemDefinition { .. } => report.with_label_opt(
                    self.location,
                    "remove this definition, or use `parse_mixed()`",
                ),
                ExecutableBuildError::AmbiguousAnonymousOperation { .. } => {
                    report.with_label_opt(self.location, "provide a name for this definition");
                    report.with_help(
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
                    report.with_label_opt(
                        *previous_location,
                        format_args!("previous definition of `{name}` here"),
                    );
                    report.with_label_opt(self.location, format_args!("`{name}` redefined here"));
                }
                ExecutableBuildError::UndefinedRootOperation { operation_type, .. } => {
                    report.with_label_opt(
                        self.location,
                        format_args!(
                            "`{operation_type}` is not defined in the schema and is therefore not supported"
                        ),
                    );
                    report.with_help(format_args!(
                        "consider defining a `{operation_type}` root operation type in your schema"
                    ))
                }
                ExecutableBuildError::UndefinedTypeInNamedFragmentTypeCondition { .. } => {
                    report.with_label_opt(self.location, "type condition here")
                }
                ExecutableBuildError::UndefinedTypeInInlineFragmentTypeCondition {
                    path, ..
                } => {
                    report.with_label_opt(self.location, "type condition here");
                    report.with_note(format_args!("path to the inline fragment: `{path} → ...`"))
                }
                ExecutableBuildError::SubselectionOnScalarType { path, .. }
                | ExecutableBuildError::SubselectionOnEnumType { path, .. } => {
                    report.with_label_opt(self.location, "remove subselections here");
                    report.with_note(format_args!("path to the field: `{path}`"))
                }
                ExecutableBuildError::UndefinedField {
                    field_name,
                    type_name,
                    path,
                    ..
                } => {
                    report.with_label_opt(
                        self.location,
                        format_args!("field `{field_name}` selected here"),
                    );
                    report.with_label_opt(
                        type_name.location(),
                        format_args!("type `{type_name}` defined here"),
                    );
                    report.with_note(format_args!("path to the field: `{path}`"))
                }
            },
        }
    }
}

impl Diagnostic<&'_ DiagnosticData> {
    /// Get the line and column number where this diagnostic was raised.
    pub fn get_line_column(&self) -> Option<GraphQLLocation> {
        GraphQLLocation::from_node(&self.sources, self.error.location)
    }

    /// Get serde_json serialisable version of the current diagnostic.
    pub fn to_json(&self) -> GraphQLError {
        GraphQLError::new(
            self.error.details.to_string(),
            self.error.location,
            &self.sources,
        )
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
    ) -> impl Iterator<Item = Diagnostic<&'_ DiagnosticData>> + DoubleEndedIterator + ExactSizeIterator
    {
        self.diagnostics_data
            .iter()
            .map(|data| data.to_diagnostic(&self.sources))
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

/// Use Debug formatting to output with colors: `format!("{diagnostics:?}")`
impl fmt::Display for DiagnosticList {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for diagnostic in self.iter() {
            fmt::Display::fmt(&diagnostic, f)?
        }
        Ok(())
    }
}

/// Use Display formatting to output without colors: `format!("{diagnostics}")`
impl fmt::Debug for DiagnosticList {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for diagnostic in self.iter() {
            fmt::Debug::fmt(&diagnostic, f)?
        }
        Ok(())
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
