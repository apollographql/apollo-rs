use std::{fmt, sync::Arc};

use crate::database::hir::{DirectiveLocation, HirNodeLocation};
use crate::database::{InputDatabase, SourceCache};
use crate::FileId;
use thiserror::Error;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct DiagnosticLocation {
    file_id: FileId,
    offset: usize,
    length: usize,
}

impl ariadne::Span for DiagnosticLocation {
    type SourceId = FileId;
    fn source(&self) -> &FileId {
        &self.file_id
    }
    fn start(&self) -> usize {
        self.offset
    }
    fn end(&self) -> usize {
        self.offset + self.length
    }
}

impl DiagnosticLocation {
    pub fn file_id(&self) -> FileId {
        self.file_id
    }
    pub fn offset(&self) -> usize {
        self.offset
    }
    pub fn node_len(&self) -> usize {
        self.length
    }
}

impl From<(FileId, rowan::TextRange)> for DiagnosticLocation {
    fn from((file_id, range): (FileId, rowan::TextRange)) -> Self {
        Self {
            file_id,
            offset: range.start().into(),
            length: range.len().into(),
        }
    }
}

impl From<(FileId, usize, usize)> for DiagnosticLocation {
    fn from((file_id, offset, length): (FileId, usize, usize)) -> Self {
        Self {
            file_id,
            offset,
            length,
        }
    }
}

impl From<HirNodeLocation> for DiagnosticLocation {
    fn from(location: HirNodeLocation) -> Self {
        Self {
            file_id: location.file_id(),
            offset: location.offset(),
            length: location.node_len(),
        }
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Label {
    pub location: DiagnosticLocation,
    pub text: String,
}
impl Label {
    pub fn new(location: impl Into<DiagnosticLocation>, text: impl Into<String>) -> Self {
        Self {
            location: location.into(),
            text: text.into(),
        }
    }
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub struct ApolloDiagnostic {
    cache: Arc<SourceCache>,
    pub location: DiagnosticLocation,
    pub labels: Vec<Label>,
    pub help: Option<String>,
    pub data: Box<DiagnosticData>,
}

impl ApolloDiagnostic {
    pub fn new<DB: InputDatabase + ?Sized>(
        db: &DB,
        location: DiagnosticLocation,
        data: DiagnosticData,
    ) -> Self {
        Self {
            cache: db.source_cache(),
            location,
            labels: vec![],
            help: None,
            data: Box::new(data),
        }
    }

    pub fn help(self, help: impl Into<String>) -> Self {
        Self {
            help: Some(help.into()),
            ..self
        }
    }

    pub fn labels(self, labels: impl Into<Vec<Label>>) -> Self {
        Self {
            labels: labels.into(),
            ..self
        }
    }

    pub fn label(mut self, label: Label) -> Self {
        self.labels.push(label);
        self
    }
}

impl fmt::Display for ApolloDiagnostic {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut buf = std::io::Cursor::new(Vec::<u8>::new());
        self.to_report()
            .write(self.cache.as_ref(), &mut buf)
            .unwrap();
        writeln!(f, "{}", std::str::from_utf8(&buf.into_inner()).unwrap())
    }
}

/// Structured data about a diagnostic.
#[derive(Debug, Error, Clone, Hash, PartialEq, Eq)]
#[non_exhaustive]
pub enum DiagnosticData {
    #[error("syntax error: {message}")]
    SyntaxError { message: String },
    #[error("limit exceeded: {message}")]
    LimitExceeded { message: String },
    #[error("expected identifier")]
    MissingIdent,
    #[error(
        "executable documents can only contain executable definitions, {}",
        name.as_ref().map(|name| format!("but `{name}` is a(n) {kind}"))
            .unwrap_or_else(|| format!("got {kind}"))
    )]
    ExecutableDefinition {
        name: Option<String>,
        kind: &'static str,
    },
    #[error("{ty} `{name}` is defined multiple times")]
    UniqueDefinition {
        ty: &'static str,
        name: String,
        original_definition: DiagnosticLocation,
        redefined_definition: DiagnosticLocation,
    },
    #[error("argument `{name}` is defined multiple times")]
    UniqueArgument {
        name: String,
        original_definition: DiagnosticLocation,
        redefined_definition: DiagnosticLocation,
    },
    #[error("value `{name}` is defined multiple times")]
    UniqueInputValue {
        name: String,
        original_value: DiagnosticLocation,
        redefined_value: DiagnosticLocation,
    },
    #[error("enum member `{name}` is defined multiple times in `{coordinate}`")]
    UniqueEnumValue {
        name: String,
        coordinate: String,
        original_definition: DiagnosticLocation,
        redefined_definition: DiagnosticLocation,
    },
    #[error("subscription operation has multiple root fields")]
    SingleRootField {
        // TODO(goto-bus-stop) if we keep this it should be a vec of the field names or nodes i think.
        // Else just remove as the labeling is done separately.
        fields: usize,
        subscription: DiagnosticLocation,
    },
    #[error("operation type {ty} is not defined")]
    UnsupportedOperation {
        // current operation type: subscription, mutation, query
        ty: &'static str,
    },
    #[error("cannot query field `{field}` on type `{ty}`")]
    UndefinedField {
        /// Field name
        field: String,
        /// Type name being queried
        ty: String,
    },
    #[error("the argument `{name}` is not supported by {coordinate}")]
    UndefinedArgument { name: String, coordinate: String },
    #[error("type `{name}` is not defined")]
    UndefinedDefinition {
        /// Name of the type not in scope
        name: String,
    },
    #[error("directive `@{name}` is not defined")]
    UndefinedDirective {
        /// Name of the missing directive
        name: String,
    },
    #[error("variable `{name}` is not defined")]
    UndefinedVariable {
        /// Name of the variable not in scope
        name: String,
    },
    #[error("fragment `{name}` is not defined")]
    UndefinedFragment {
        /// Name of the fragment not in scope
        name: String,
    },
    #[error("value `{value}` does not exist on `{definition}` type")]
    UndefinedValue {
        /// Value of the enum that doesn't exist
        value: String,
        /// type definition
        definition: String,
    },
    #[error("type extension for `{name}` is the wrong kind")]
    WrongTypeExtension {
        /// Name of the type being extended
        name: String,
        /// Location of the original definition. This may be None when extending a builtin GraphQL type.
        definition: DiagnosticLocation,
        /// Location of the extension
        extension: DiagnosticLocation,
    },
    #[error("directive definition `{name}` references itself")]
    RecursiveDirectiveDefinition { name: String },
    #[error("interface `{name}` implements itself")]
    RecursiveInterfaceDefinition { name: String },
    #[error("input object `{name}` references itself")]
    RecursiveInputObjectDefinition { name: String },
    #[error("fragment `{name}` references itself")]
    RecursiveFragmentDefinition { name: String },
    #[error("enum value `{value}` does not have a conventional all-caps name")]
    CapitalizedValue { value: String },
    #[error("field `{field}` is declared multiple times")]
    UniqueField {
        /// Name of the non-unique field.
        field: String,
        original_definition: DiagnosticLocation,
        redefined_definition: DiagnosticLocation,
    },
    #[error("type `{name}` does not satisfy interface `{interface}` because it is missing field `{field}`")]
    MissingField {
        name: String,
        interface: String,
        // current field that should be defined
        field: String,
    },
    #[error("required argument `{coordinate}` is not provided")]
    RequiredArgument { coordinate: String },
    #[error("type `{ty}` implements interface `{interface}` multiple times")]
    DuplicateImplementsInterface { ty: String, interface: String },
    // XXX(@goto-bus-stop): This error message would be better if it listed which other interface
    // is responsible for `missing_interface` being required. It would require code changes in
    // validation to pass that information on, so not doing it in 0.11.x--we should do it in 1.0 :)
    #[error("type `{ty}` must implement `{missing_interface}`")]
    TransitiveImplementedInterfaces {
        /// Name of the affected type
        ty: String,
        // interface that should be defined
        missing_interface: String,
    },
    #[error("`{name}` field does not return an output type")]
    OutputType {
        // field name
        name: String,
        // field type
        ty: &'static str,
    },
    #[error("type `{ty}` for input field or argument `{name}` is not an input type")]
    InputType {
        /// Field name.
        name: String,
        /// Declared type.
        ty: String,
    },
    #[error("custom scalar `{ty}` does not have an @specifiedBy directive")]
    ScalarSpecificationURL {
        /// Name of the scalar.
        ty: String,
    },
    #[error("missing query root operation type in schema definition")]
    QueryRootOperationType,
    #[error("built-in scalars must be omitted for brevity")]
    BuiltInScalarDefinition,
    #[error("type `{ty}` for variable `${name}` is not an input type")]
    VariableInputType {
        /// Variable name.
        name: String,
        /// Declared type.
        ty: String,
    },
    #[error("variable `{name}` is unused")]
    UnusedVariable { name: String },
    #[error("field `{name}` does not return an object type")]
    ObjectType {
        // union member
        name: String,
        // actual type
        ty: &'static str,
    },
    #[error("directive `@{name}` can not be used on {dir_loc}")]
    UnsupportedDirectiveLocation {
        /// name of the directive
        name: String,
        /// current location where the directive is used
        dir_loc: DirectiveLocation,
        /// The source location where the directive that's being used was defined.
        directive_def: DiagnosticLocation,
    },
    #[error("`{value}` cannot be assigned to type `{ty}`")]
    UnsupportedValueType {
        // input value
        value: String,
        // defined type
        ty: String,
    },
    #[error("int cannot represent non 32-bit signed integer value")]
    IntCoercionError {
        /// The int value that cannot be coerced
        value: String,
    },
    #[error("non-repeatable directive {name} can only be used once per location")]
    UniqueDirective {
        /// Name of the non-unique directive.
        name: String,
        original_call: DiagnosticLocation,
        conflicting_call: DiagnosticLocation,
    },
    #[error("subscription operation uses introspection field `{field}` as a root field")]
    IntrospectionField {
        /// Name of the field
        field: String,
    },
    #[error("field `{field}` has a subselection but its type `{ty}` is not a composite type")]
    DisallowedSubselection {
        /// Name of the field
        field: String,
        /// Name of the type
        ty: String,
    },
    #[error("field `{field}` selects a composite type `{ty}` but does not have a subselection")]
    MissingSubselection {
        /// Name of the field
        field: String,
        /// Name of the type
        ty: String,
    },
    #[error("operation selects different types into the same field name `{field}`")]
    ConflictingField {
        /// Name of the non-unique field.
        field: String,
        original_selection: DiagnosticLocation,
        redefined_selection: DiagnosticLocation,
    },
    #[error("fragments must be specified on types that exist in the schema")]
    InvalidFragment {
        /// Name of the type on which the fragment is declared
        ty: Option<String>,
    },
    #[error("type condition `{ty}` is not a composite type")]
    InvalidFragmentTarget {
        /// Name of the type on which the fragment is declared
        ty: String,
    },
    #[error(
        "{} cannot be applied to type `{type_name}`",
        name.as_ref().map(|name| format!("fragment `{name}`"))
            .unwrap_or_else(|| "anonymous fragment".to_string()),
    )]
    InvalidFragmentSpread {
        /// Fragment name or None if it's an inline fragment
        name: Option<String>,
        /// Type name the fragment is being applied to
        type_name: String,
    },
    #[error("fragment `{name}` is unused")]
    UnusedFragment {
        /// Name of the fragment
        name: String,
    },
    #[error(
        "variable `{var_name}` cannot be used for argument `{arg_name}` as their types mismatch"
    )]
    DisallowedVariableUsage {
        /// Name of the variable being used in an argument
        var_name: String,
        /// Name of the argument where variable is used
        arg_name: String,
    },
}

impl DiagnosticData {
    pub fn is_error(&self) -> bool {
        !self.is_warning() && !self.is_advice()
    }
    pub fn is_warning(&self) -> bool {
        matches!(self, Self::CapitalizedValue { .. })
    }
    pub fn is_advice(&self) -> bool {
        matches!(self, Self::ScalarSpecificationURL { .. })
    }
}

impl From<Label> for ariadne::Label<DiagnosticLocation> {
    fn from(label: Label) -> Self {
        Self::new(label.location).with_message(label.text)
    }
}

impl ApolloDiagnostic {
    pub fn to_report(&self) -> ariadne::Report<'static, DiagnosticLocation> {
        use ariadne::{ColorGenerator, Report, ReportKind};

        let severity = if self.data.is_advice() {
            ReportKind::Advice
        } else if self.data.is_warning() {
            ReportKind::Warning
        } else {
            ReportKind::Error
        };
        let mut colors = ColorGenerator::new();
        let mut builder = Report::build(severity, self.location.file_id(), self.location.offset())
            .with_message(&self.data);
        builder.add_labels(
            self.labels
                .iter()
                .map(|label| ariadne::Label::from(label.clone()).with_color(colors.next())),
        );
        if let Some(help) = &self.help {
            builder = builder.with_help(help);
        }
        builder.finish()
    }
}
