use std::fmt;

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
    cache: SourceCache,
    pub location: DiagnosticLocation,
    pub labels: Vec<Label>,
    pub help: Option<String>,
    pub data: DiagnosticData,
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
            data,
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
        self.to_report().write(&self.cache, &mut buf).unwrap();
        writeln!(f, "{}", std::str::from_utf8(&buf.into_inner()).unwrap())
    }
}

/// Structured data about a diagnostic.
#[derive(Debug, Error, Clone, Hash, PartialEq, Eq)]
#[non_exhaustive]
pub enum DiagnosticData {
    #[error("syntax error: {message}")]
    SyntaxError { message: String },
    #[error("expected identifier")]
    MissingIdent,
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
    #[error("cannot query `{field}` field")]
    UndefinedField {
        /// Field name
        field: String,
    },
    #[error("the argument `{name}` is not supported")]
    UndefinedArgument { name: String },
    #[error("cannot find type `{name}` in this document")]
    UndefinedDefinition {
        /// Name of the type not in scope
        name: String,
    },
    #[error("type extension for `{name}` is the wrong kind")]
    WrongTypeExtension {
        /// Name of the type being extended
        name: String,
        /// Location of the original definition. This may be None when extending a builtin GraphQL type.
        definition: Option<DiagnosticLocation>,
        /// Location of the extension
        extension: DiagnosticLocation,
    },
    #[error("{name} directive definition cannot reference itself")]
    RecursiveDefinition { name: String },
    #[error("interface {name} cannot implement itself")]
    RecursiveInterfaceDefinition { name: String },
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
    #[error("`${name}` variable must be of an input type")]
    InputType {
        // variable name
        name: String,
        // variable type
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
        directive_def: Option<DiagnosticLocation>,
    },
    #[error("subscription operations can not have an introspection field as a root field")]
    IntrospectionField {
        /// Name of the field
        field: String,
    },
    #[error("operation must not select different types as the same name `{field}`")]
    ConflictingSelection {
        /// Name of the non-unique field.
        field: String,
        original_selection: DiagnosticLocation,
        redefined_selection: DiagnosticLocation,
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
