use crate::ast::DirectiveLocation;
use crate::FileId;
use crate::NodeLocation;
use std::fmt;
use thiserror::Error;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub(crate) struct Label {
    pub location: Option<NodeLocation>,
    pub text: String,
}
impl Label {
    pub fn new(location: Option<NodeLocation>, text: impl Into<String>) -> Self {
        Self {
            location,
            text: text.into(),
        }
    }
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub(crate) struct ApolloDiagnostic {
    pub location: Option<NodeLocation>,
    pub labels: Vec<Label>,
    pub help: Option<String>,
    pub data: Box<DiagnosticData>,
}

impl ApolloDiagnostic {
    pub fn new<DB: ?Sized>(_db: &DB, location: Option<NodeLocation>, data: DiagnosticData) -> Self {
        Self {
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
        self.data.fmt(f)
    }
}

/// Structured data about a diagnostic.
#[derive(Debug, Error, Clone, Hash, PartialEq, Eq)]
#[non_exhaustive]
pub(crate) enum DiagnosticData {
    #[error("the {ty} `{name}` is defined multiple times in the document")]
    UniqueDefinition {
        ty: &'static str,
        name: String,
        original_definition: Option<NodeLocation>,
        redefined_definition: Option<NodeLocation>,
    },
    #[error("the argument `{name}` is defined multiple times")]
    UniqueArgument {
        name: String,
        original_definition: Option<NodeLocation>,
        redefined_definition: Option<NodeLocation>,
    },
    #[error("the value `{name}` is defined multiple times")]
    UniqueInputValue {
        name: String,
        original_value: Option<NodeLocation>,
        redefined_value: Option<NodeLocation>,
    },
    #[error("subscription operations can only have one root field")]
    SingleRootField {
        // TODO(goto-bus-stop) if we keep this it should be a vec of the field names or nodes i think.
        // Else just remove as the labeling is done separately.
        fields: usize,
        subscription: Option<NodeLocation>,
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
    #[error("type does not satisfy interface `{interface}`: missing field `{field}`")]
    MissingInterfaceField { interface: String, field: String },
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
        directive_def: Option<NodeLocation>,
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
    #[error("float cannot represent non-finite 64-bit floating point value")]
    FloatCoercionError {
        /// The float value that cannot be coerced
        value: String,
    },
    #[error("non-repeatable directive {name} can only be used once per location")]
    UniqueDirective {
        /// Name of the non-unique directive.
        name: String,
        original_call: Option<NodeLocation>,
        conflicting_call: Option<NodeLocation>,
    },
    #[error("subscription operations can not have an introspection field as a root field")]
    IntrospectionField {
        /// Name of the field
        field: String,
    },
    #[error("interface, union and object types must have a subselection set")]
    MissingSubselection,
    #[error("operation must not select different types using the same field name `{field}`")]
    ConflictingField {
        /// Name of the non-unique field.
        field: String,
        original_selection: Option<NodeLocation>,
        redefined_selection: Option<NodeLocation>,
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

impl ApolloDiagnostic {
    pub(crate) fn to_report(
        &self,
        config: ariadne::Config,
    ) -> ariadne::Report<'static, NodeLocation> {
        use ariadne::{ColorGenerator, Report, ReportKind};

        let severity = if self.data.is_advice() {
            ReportKind::Advice
        } else if self.data.is_warning() {
            ReportKind::Warning
        } else {
            ReportKind::Error
        };
        let mut colors = ColorGenerator::new();
        let (id, offset) = if let Some(location) = self.location {
            (location.file_id(), location.offset())
        } else {
            (FileId::NONE, 0)
        };
        let mut builder = Report::build(severity, id, offset)
            .with_config(config)
            .with_message(&self.data);
        builder.add_labels(self.labels.iter().filter_map(|label| {
            label.location.map(|loc| {
                ariadne::Label::new(loc)
                    .with_message(&label.text)
                    .with_color(colors.next())
            })
        }));
        if let Some(help) = &self.help {
            builder = builder.with_help(help);
        }
        builder.finish()
    }
}
