use std::{fmt, sync::Arc};

use crate::database::hir::{DirectiveLocation, HirNodeLocation};
use crate::database::{InputDatabase, SourceCache};
use crate::FileId;
use thiserror::Error;

/// A source location (line + column) for a GraphQL error.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct GraphQLLocation {
    /// The line number for this location, starting at 1 for the first line.
    pub line: usize,
    /// The column number for this location, starting at 1 and counting characters (Unicode Scalar
    /// Values) like [str::chars].
    pub column: usize,
}

/// A serializable GraphQL error.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct GraphQLError {
    /// The error message.
    pub message: String,

    /// Locations relevant to the error, if any.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub locations: Vec<GraphQLLocation>,

    /// The operation path where the error occurred.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<Vec<serde_json::Value>>,

    /// Additional freeform data.
    #[serde(default, skip_serializing_if = "serde_json::Map::is_empty")]
    pub extensions: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct DiagnosticLocation {
    file_id: FileId,
    offset: usize,
    length: usize,
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

    /// Get the line and column number where this diagnostic was raised.
    pub fn get_line_column(&self) -> Option<GraphQLLocation> {
        self.cache
            .get_line_column(self.location.file_id, self.location.offset)
            // Make 1-indexed
            .map(|(line, column)| GraphQLLocation { line: line + 1, column: column + 1 })
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
    #[error("executable documents must not contain {kind}")]
    ExecutableDefinition { kind: &'static str },
    #[error("the {ty} `{name}` is defined multiple times in the document")]
    UniqueDefinition {
        ty: &'static str,
        name: String,
        original_definition: DiagnosticLocation,
        redefined_definition: DiagnosticLocation,
    },
    #[error("the argument `{name}` is defined multiple times")]
    UniqueArgument {
        name: String,
        original_definition: DiagnosticLocation,
        redefined_definition: DiagnosticLocation,
    },
    #[error("the value `{name}` is defined multiple times")]
    UniqueInputValue {
        name: String,
        original_value: DiagnosticLocation,
        redefined_value: DiagnosticLocation,
    },
    #[error("subscription operations can only have one root field")]
    SingleRootField {
        // TODO(goto-bus-stop) if we keep this it should be a vec of the field names or nodes i think.
        // Else just remove as the labeling is done separately.
        fields: usize,
        subscription: DiagnosticLocation,
    },
    #[error("{ty} root operation type is not defined")]
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
    #[error("the argument `{name}` is not supported")]
    UndefinedArgument { name: String },
    #[error("cannot find type `{name}` in this document")]
    UndefinedDefinition {
        /// Name of the type not in scope
        name: String,
    },
    #[error("cannot find directive `{name}` in this document")]
    UndefinedDirective {
        /// Name of the missing directive
        name: String,
    },
    #[error("variable `{name}` is not defined")]
    UndefinedVariable {
        /// Name of the variable not in scope
        name: String,
    },
    #[error("cannot find fragment `{name}` in this document")]
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
    #[error("`{name}` directive definition cannot reference itself")]
    RecursiveDirectiveDefinition { name: String },
    #[error("interface {name} cannot implement itself")]
    RecursiveInterfaceDefinition { name: String },
    #[error("`{name}` input object cannot reference itself")]
    RecursiveInputObjectDefinition { name: String },
    #[error("`{name}` fragment cannot reference itself")]
    RecursiveFragmentDefinition { name: String },
    #[error("values in an Enum Definition should be capitalized")]
    CapitalizedValue { value: String },
    #[error("fields must be unique in a definition")]
    UniqueField {
        /// Name of the non-unique field.
        field: String,
        original_definition: DiagnosticLocation,
        redefined_definition: DiagnosticLocation,
    },
    #[error("missing `{field}` field")]
    MissingField {
        // current field that should be defined
        field: String,
    },
    #[error("the required argument `{name}` is not provided")]
    RequiredArgument { name: String },
    #[error("type `{ty}` can only implement interface `{interface}` once")]
    DuplicateImplementsInterface { ty: String, interface: String },
    #[error(
        "Transitively implemented interfaces must also be defined on an implementing interface or object"
    )]
    TransitiveImplementedInterfaces {
        // interface that should be defined
        missing_interface: String,
    },
    #[error("`{name}` field must return an output type")]
    OutputType {
        // field name
        name: String,
        // field type
        ty: &'static str,
    },
    #[error("`{name}` field must be of an input type")]
    InputType {
        /// Field name.
        name: String,
        /// The kind of type that the field is declared with.
        ty: &'static str,
    },
    #[error(
        "custom scalars should provide a scalar specification URL via the @specifiedBy directive"
    )]
    ScalarSpecificationURL,
    #[error("missing query root operation type in schema definition")]
    QueryRootOperationType,
    #[error("built-in scalars must be omitted for brevity")]
    BuiltInScalarDefinition,
    #[error("`${name}` variable must be of an input type")]
    VariableInputType {
        /// Varialbe name.
        name: String,
        /// The kind of type that the variable is declared with.
        ty: &'static str,
    },
    #[error("unused variable: `{name}`")]
    UnusedVariable { name: String },
    #[error("`{name}` field must return an object type")]
    ObjectType {
        // union member
        name: String,
        // actual type
        ty: &'static str,
    },
    #[error("{name} directive is not supported for {dir_loc} location")]
    UnsupportedLocation {
        /// current directive definition
        name: String,
        /// current location where the directive is used
        dir_loc: DirectiveLocation,
        /// The source location where the directive that's being used was defined.
        directive_def: DiagnosticLocation,
    },
    #[error("{ty} cannot be represented by a {value} value")]
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
    #[error("subscription operations can not have an introspection field as a root field")]
    IntrospectionField {
        /// Name of the field
        field: String,
    },
    #[error("subselection set for scalar and enum types must be empty")]
    DisallowedSubselection,
    #[error("interface, union and object types must have a subselection set")]
    MissingSubselection,
    #[error("operation must not select different types using the same field name `{field}`")]
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
    #[error("fragments can not be declared on primitive types")]
    InvalidFragmentTarget {
        /// Name of the type on which the fragment is declared
        ty: String,
    },
    #[error("fragment cannot be applied to this type")]
    InvalidFragmentSpread {
        /// Fragment name or None if it's an inline fragment
        name: Option<String>,
        /// Type name the fragment is being applied to
        type_name: String,
    },
    #[error("fragment `{name}` must be used in an operation")]
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
        matches!(self, Self::ScalarSpecificationURL)
    }
}

type AriadneSpan = (FileId, std::ops::Range<usize>);
impl ApolloDiagnostic {
    fn map_location(&self, location: DiagnosticLocation) -> AriadneSpan {
        let id = location.file_id();
        let source = self.cache.get_source(id).unwrap();
        let start = source.map_index(location.offset);
        let end = source.map_index(location.offset + location.length);
        (id, start..end)
    }

    pub fn to_report(&self) -> ariadne::Report<'static, AriadneSpan> {
        use ariadne::{ColorGenerator, Report, ReportKind};

        let severity = if self.data.is_advice() {
            ReportKind::Advice
        } else if self.data.is_warning() {
            ReportKind::Warning
        } else {
            ReportKind::Error
        };

        let span = self.map_location(self.location);

        let mut colors = ColorGenerator::new();
        let mut builder =
            Report::build(severity, self.location.file_id(), span.1.start).with_message(&self.data);
        builder.add_labels(self.labels.iter().map(|label| {
            ariadne::Label::new(self.map_location(label.location))
                .with_message(&label.text)
                .with_color(colors.next())
        }));
        if let Some(help) = &self.help {
            builder = builder.with_help(help);
        }
        builder.finish()
    }

    pub fn to_json(&self) -> GraphQLError {
        let mut locations = vec![];

        if let Some(location) = self.get_line_column() {
            locations.push(location);
        }

        GraphQLError {
            message: self.data.to_string(),
            locations,
            path: Default::default(),
            extensions: Default::default(),
        }
    }
}
