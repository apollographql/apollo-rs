use crate::ast;
use crate::ast::DirectiveLocation;
use crate::ast::Name;
use crate::ast::Type;
use crate::coordinate::SchemaCoordinate;
use crate::coordinate::TypeAttributeCoordinate;
use crate::diagnostic::CliReport;
use crate::executable;
use crate::Node;
use crate::NodeLocation;
use std::fmt;
use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub(crate) struct ValidationError {
    pub location: Option<NodeLocation>,
    pub data: DiagnosticData,
}

impl ValidationError {
    pub fn new(location: Option<NodeLocation>, data: DiagnosticData) -> Self {
        Self { location, data }
    }
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.data.fmt(f)
    }
}

/// Structured data about a diagnostic.
#[derive(Debug, Error, Clone, Hash, PartialEq, Eq)]
#[non_exhaustive]
pub(crate) enum DiagnosticData {
    #[error("the variable `${name}` is declared multiple times")]
    UniqueVariable {
        name: Name,
        original_definition: Option<NodeLocation>,
        redefined_definition: Option<NodeLocation>,
    },
    #[error("the argument `{name}` is provided multiple times")]
    UniqueArgument {
        name: Name,
        original_definition: Option<NodeLocation>,
        redefined_definition: Option<NodeLocation>,
    },
    #[error("the value `{name}` is defined multiple times")]
    UniqueInputValue {
        name: Name,
        original_definition: Option<NodeLocation>,
        redefined_definition: Option<NodeLocation>,
    },
    #[error("the argument `{name}` is not supported by `{coordinate}`")]
    UndefinedArgument {
        name: Name,
        coordinate: SchemaCoordinate,
        definition_location: Option<NodeLocation>,
    },
    #[error("cannot find type `{name}` in this document")]
    UndefinedDefinition {
        /// Name of the type not in scope
        name: Name,
    },
    #[error("cannot find directive `@{name}` in this document")]
    UndefinedDirective {
        /// Name of the missing directive
        name: Name,
    },
    #[error("variable `${name}` is not defined")]
    UndefinedVariable {
        /// Name of the variable not in scope
        name: Name,
    },
    #[error("cannot find fragment `{name}` in this document")]
    UndefinedFragment {
        /// Name of the fragment not in scope
        name: Name,
    },
    #[error("value `{value}` does not exist on `{definition}`")]
    UndefinedEnumValue {
        /// Value of the enum value that doesn't exist
        value: Name,
        /// Name of the enum
        definition: Name,
        definition_location: Option<NodeLocation>,
    },
    #[error("field `{value}` does not exist on `{definition}`")]
    UndefinedInputValue {
        /// Value of the input object field that doesn't exist
        value: Name,
        /// Name of the input object type
        definition: Name,
        definition_location: Option<NodeLocation>,
    },
    #[error("type `{name}` does not satisfy interface `{interface}`: missing field `{field}`")]
    MissingInterfaceField {
        name: Name,
        /// Location of the `implements XYZ` of the interface
        implements_location: Option<NodeLocation>,
        interface: Name,
        field: Name,
        /// Location of the definition of the field in the interface
        field_location: Option<NodeLocation>,
    },
    #[error("the required argument `{coordinate}` is not provided")]
    RequiredArgument {
        name: Name,
        coordinate: SchemaCoordinate,
        definition_location: Option<NodeLocation>,
    },
    #[error("the required field `{coordinate}` is not provided")]
    RequiredField {
        name: Name,
        coordinate: TypeAttributeCoordinate,
        definition_location: Option<NodeLocation>,
    },
    #[error(
        "interface `{interface}` declares that it implements `{via_interface}`, but to do so it must also implement `{missing_interface}`"
    )]
    TransitiveImplementedInterfaces {
        /// Name of the interface definition
        interface: Name,
        /// Super interface that declares the implementation
        via_interface: Name,
        /// Source location where the super interface declares the implementation
        transitive_interface_location: Option<NodeLocation>,
        /// Interface that should be implemented
        missing_interface: Name,
    },
    #[error("`{name}` field must return an output type")]
    OutputType {
        /// Field name.
        name: Name,
        /// The kind of type that the field is declared with.
        describe_type: &'static str,
        type_location: Option<NodeLocation>,
    },
    #[error("`{name}` field must be of an input type")]
    InputType {
        /// Field name.
        name: Name,
        /// The kind of type that the field is declared with.
        describe_type: &'static str,
        type_location: Option<NodeLocation>,
    },
    #[error("`${name}` variable must be of an input type")]
    VariableInputType {
        /// Variable name.
        name: Name,
        /// The kind of type that the variable is declared with.
        describe_type: &'static str,
        type_location: Option<NodeLocation>,
    },
    #[error("missing query root operation type in schema definition")]
    QueryRootOperationType,
    #[error("unused variable: `${name}`")]
    UnusedVariable { name: Name },
    #[error("`{name}` field must return an object type")]
    RootOperationObjectType {
        /// Name of the root operation type
        name: Name,
        /// Category of the type
        describe_type: &'static str,
    },
    #[error("union member `{name}` must be an object type")]
    UnionMemberObjectType {
        /// Name of the type in the union
        name: Name,
        /// Category of the type
        describe_type: &'static str,
    },
    #[error("{name} directive is not supported for {location} location")]
    UnsupportedLocation {
        /// Name of the directive
        name: Name,
        /// The location where the directive is attempted to be used
        location: DirectiveLocation,
        /// Locations that *are* valid for this directive
        valid_locations: Vec<DirectiveLocation>,
        /// The source location where the directive that's being used was defined.
        definition_location: Option<NodeLocation>,
    },
    #[error("expected value of type {ty}, found {describe_value_type}")]
    UnsupportedValueType {
        /// The kind of value provided.
        describe_value_type: &'static str,
        /// Expected concrete type
        ty: String,
        definition_location: Option<NodeLocation>,
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
        name: Name,
        original_application: Option<NodeLocation>,
    },
    #[error("interface, union and object types must have a subselection set")]
    MissingSubselection {
        coordinate: TypeAttributeCoordinate,
        describe_type: &'static str,
    },
    #[error(
        "{} must have a composite type in its type condition",
        fragment_name_or_inline(name)
    )]
    InvalidFragmentTarget {
        /// Name of the fragment, None if an inline fragment.
        name: Option<Name>,
        /// Name of the type on which the fragment is declared
        ty: Name,
    },
    #[error(
        "{} with type condition `{type_condition}` cannot be applied to `{type_name}`",
        fragment_name_or_inline(name)
    )]
    InvalidFragmentSpread {
        /// Fragment name or None if it's an inline fragment
        name: Option<Name>,
        /// Type name the fragment is being applied to
        type_name: Name,
        type_condition: Name,
        /// Source location where the fragment is defined
        fragment_location: Option<NodeLocation>,
        /// Source location of the type the fragment is being applied to.
        type_location: Option<NodeLocation>,
    },
    #[error("fragment `{name}` must be used in an operation")]
    UnusedFragment {
        /// Name of the fragment
        name: Name,
    },
    #[error(
        "variable `${variable}` of type `{variable_type}` cannot be used for argument `{argument}` of type `{argument_type}`"
    )]
    DisallowedVariableUsage {
        /// Name of the variable being used in an argument
        variable: Name,
        variable_type: Type,
        variable_location: Option<NodeLocation>,
        /// Name of the argument where variable is used
        argument: Name,
        argument_type: Type,
        argument_location: Option<NodeLocation>,
    },
    #[error("`{name}` directive definition cannot reference itself")]
    RecursiveDirectiveDefinition {
        name: Name,
        trace: Vec<Node<ast::Directive>>,
    },
    #[error("interface {name} cannot implement itself")]
    RecursiveInterfaceDefinition { name: Name },
    #[error("`{name}` input object cannot reference itself")]
    RecursiveInputObjectDefinition {
        name: Name,
        trace: Vec<Node<ast::InputValueDefinition>>,
    },
    #[error("`{name}` fragment cannot reference itself")]
    RecursiveFragmentDefinition {
        /// Source location of just the "fragment FragName" part.
        head_location: Option<NodeLocation>,
        name: Name,
        trace: Vec<Node<executable::FragmentSpread>>,
    },
    #[error("`{name}` contains too much nesting")]
    DeeplyNestedType {
        name: Name,
        describe_type: &'static str,
    },
    #[error("too much recursion")]
    RecursionError {},
}

impl ValidationError {
    pub(crate) fn report(&self, report: &mut CliReport) {
        match &self.data {
            DiagnosticData::UniqueVariable {
                name,
                original_definition,
                redefined_definition,
            } => {
                report.with_label_opt(
                    *original_definition,
                    format_args!("previous definition of `${name}` here"),
                );
                report.with_label_opt(
                    *redefined_definition,
                    format_args!("`${name}` defined again here"),
                );
            }
            DiagnosticData::UniqueArgument {
                name,
                original_definition,
                redefined_definition,
            } => {
                report.with_label_opt(
                    *original_definition,
                    format_args!("previously provided `{name}` here"),
                );
                report.with_label_opt(
                    *redefined_definition,
                    format_args!("`{name}` provided again here"),
                );
                report.with_help(format_args!(
                    "`{name}` argument must only be provided once."
                ));
            }
            DiagnosticData::UniqueInputValue {
                name,
                original_definition,
                redefined_definition,
            } => {
                report.with_label_opt(
                    *original_definition,
                    format_args!("previous definition of `{name}` here"),
                );
                report.with_label_opt(
                    *redefined_definition,
                    format_args!("`{name}` defined again here"),
                );
                report.with_help(format_args!(
                    "`{name}` must only be defined once in this argument list or input object definition."
                ));
            }
            DiagnosticData::UndefinedArgument {
                coordinate,
                definition_location,
                ..
            } => {
                report.with_label_opt(self.location, "argument by this name not found");
                report.with_label_opt(
                    *definition_location,
                    format_args!("{coordinate} defined here"),
                );
            }
            DiagnosticData::RequiredArgument {
                name,
                coordinate: _,
                definition_location,
            } => {
                report.with_label_opt(
                    self.location,
                    format_args!("missing value for argument `{name}`"),
                );
                report.with_label_opt(*definition_location, "argument defined here");
            }
            DiagnosticData::RequiredField {
                name,
                coordinate: _,
                definition_location,
            } => {
                report.with_label_opt(
                    self.location,
                    format_args!("missing value for field `{name}`"),
                );
                report.with_label_opt(*definition_location, "field defined here");
            }
            DiagnosticData::UndefinedDefinition { .. } => {
                report.with_label_opt(self.location, "not found in this scope");
            }
            DiagnosticData::UndefinedDirective { .. } => {
                report.with_label_opt(self.location, "directive not defined");
            }
            DiagnosticData::UndefinedVariable { .. } => {
                report.with_label_opt(self.location, "not found in this scope");
            }
            DiagnosticData::UndefinedFragment { name } => {
                report.with_label_opt(
                    self.location,
                    format_args!("fragment `{name}` is not defined"),
                );
            }
            DiagnosticData::UndefinedEnumValue {
                value: _,
                definition,
                definition_location,
            } => {
                report.with_label_opt(
                    self.location,
                    format_args!("value does not exist on `{definition}` enum"),
                );
                report.with_label_opt(*definition_location, "enum defined here");
            }
            DiagnosticData::UndefinedInputValue {
                value: _,
                definition,
                definition_location,
            } => {
                report.with_label_opt(
                    self.location,
                    format_args!("value does not exist on `{definition}` input object"),
                );
                report.with_label_opt(*definition_location, "input object defined here");
            }
            DiagnosticData::RecursiveDirectiveDefinition { name, trace } => {
                report.with_label_opt(self.location, "recursive directive definition");
                label_recursive_trace(report, trace, name, |directive| &directive.name);
            }
            DiagnosticData::RecursiveInterfaceDefinition { name } => {
                report.with_label_opt(
                    self.location,
                    format_args!("interface {name} cannot implement itself"),
                );
            }
            DiagnosticData::RecursiveInputObjectDefinition { name, trace } => {
                report.with_label_opt(self.location, "cyclical input object definition");
                label_recursive_trace(report, trace, name, |reference| &reference.name);
            }
            DiagnosticData::RecursiveFragmentDefinition {
                head_location,
                name,
                trace,
            } => {
                report.with_label_opt(
                    head_location.or(self.location),
                    "recursive fragment definition",
                );
                label_recursive_trace(report, trace, name, |reference| &reference.fragment_name);
            }
            DiagnosticData::DeeplyNestedType { describe_type, .. } => {
                report.with_label_opt(
                    self.location,
                    format_args!(
                        "references a very long chain of {describe_type}s in its definition"
                    ),
                );
            }
            DiagnosticData::MissingInterfaceField {
                name: _,
                implements_location,
                interface,
                field,
                field_location,
            } => {
                report.with_label_opt(
                    self.location,
                    format_args!("add `{field}` field to this type"),
                );
                report.with_label_opt(
                    *implements_location,
                    format_args!("implementation of interface {interface} declared here"),
                );
                report.with_label_opt(
                    *field_location,
                    format_args!("`{interface}.{field}` originally defined here"),
                );
                report.with_help(
                    "An object or interface must declare all fields required by the interfaces it implements",
                )
            }
            DiagnosticData::TransitiveImplementedInterfaces {
                interface: _,
                via_interface,
                transitive_interface_location,
                missing_interface,
            } => {
                report.with_label_opt(
                    *transitive_interface_location,
                    format!(
                        "implementation of {missing_interface} declared by {via_interface} here"
                    ),
                );
                report.with_label_opt(
                    self.location,
                    format_args!("{missing_interface} must also be implemented here"),
                );
            }
            DiagnosticData::UnusedVariable { .. } => {
                report.with_label_opt(self.location, "variable is never used");
            }
            DiagnosticData::UnusedFragment { name } => {
                report.with_label_opt(self.location, format_args!("`{name}` is defined here"));
                report.with_help(format_args!(
                    "fragment `{name}` must be used in an operation"
                ));
            }
            DiagnosticData::RootOperationObjectType {
                name: _,
                describe_type,
            } => {
                report.with_label_opt(self.location, format_args!("this is {describe_type}"));
                report.with_help("Root operation type must be an object type.");
            }
            DiagnosticData::UnionMemberObjectType {
                name: _,
                describe_type,
            } => {
                report.with_label_opt(self.location, format_args!("this is {describe_type}"));
                report.with_help("Union members must be object types.");
            }
            DiagnosticData::OutputType {
                name,
                describe_type,
                type_location,
            } => {
                report.with_label_opt(
                    type_location.or(self.location),
                    format_args!("this is {describe_type}"),
                );
                report.with_help(format!("Scalars, Objects, Interfaces, Unions and Enums are output types. Change `{name}` field to return one of these output types."));
            }
            DiagnosticData::InputType {
                name,
                describe_type,
                type_location,
            } => {
                report.with_label_opt(
                    type_location.or(self.location),
                    format_args!("this is {describe_type}"),
                );
                report.with_help(format!("Scalars, Enums, and Input Objects are input types. Change `{name}` field to take one of these input types."));
            }
            DiagnosticData::VariableInputType {
                name: _,
                describe_type,
                type_location,
            } => {
                report.with_label_opt(
                    type_location.or(self.location),
                    format_args!("this is {describe_type}"),
                );
                report.with_help("objects, unions, and interfaces cannot be used because variables can only be of input type");
            }
            DiagnosticData::QueryRootOperationType => {
                report.with_label_opt(
                    self.location,
                    "`query` root operation type must be defined here",
                );
            }
            DiagnosticData::UnsupportedLocation {
                name: _,
                location,
                valid_locations,
                definition_location,
            } => {
                report.with_label_opt(
                    self.location,
                    format_args!("directive cannot be used on {location}"),
                );
                report.with_label_opt(*definition_location, "directive defined here");
                report.with_help(format!(
                    "the directive must be used in a location that the service has declared support for: {}",
                    CommaSeparated(valid_locations),
                ));
            }
            DiagnosticData::UnsupportedValueType {
                describe_value_type,
                ty,
                definition_location,
            } => {
                report.with_label_opt(
                    self.location,
                    format_args!("provided value is {describe_value_type}"),
                );
                report.with_label_opt(
                    *definition_location,
                    format_args!("expected type declared here as {ty}"),
                );
            }
            DiagnosticData::IntCoercionError { .. } => {
                report.with_label_opt(self.location, "cannot be coerced to a 32-bit integer");
            }
            DiagnosticData::FloatCoercionError { .. } => {
                report.with_label_opt(self.location, "cannot be coerced to a finite 64-bit float");
            }
            DiagnosticData::UniqueDirective {
                name,
                original_application,
            } => {
                report.with_label_opt(
                    *original_application,
                    format_args!("directive `@{name}` first called here"),
                );
                report.with_label_opt(
                    self.location,
                    format_args!("directive `@{name}` called again here"),
                );
            }
            DiagnosticData::MissingSubselection {
                coordinate,
                describe_type,
            } => {
                report.with_label_opt(
                    self.location,
                    format_args!("{coordinate} is {describe_type} and must select fields"),
                );
            }
            DiagnosticData::InvalidFragmentTarget { name: _, ty } => {
                report.with_label_opt(
                    self.location,
                    format!("fragment declares unsupported type condition `{ty}`"),
                );
                report.with_help("fragments cannot be defined on enums, scalars and input objects");
            }
            DiagnosticData::InvalidFragmentSpread {
                name,
                type_name: _,
                type_condition,
                fragment_location,
                type_location,
            } => {
                if let Some(name) = name {
                    report.with_label_opt(
                        self.location,
                        format_args!("fragment `{name}` cannot be applied"),
                    );
                    // Only for named fragments: for inline fragments the type condition is right
                    // there
                    report.with_label_opt(
                        *fragment_location,
                        format_args!(
                            "fragment declared with type condition `{type_condition}` here"
                        ),
                    );
                } else {
                    report.with_label_opt(self.location, "inline fragment cannot be applied");
                }
                report.with_label_opt(
                    *type_location,
                    format!("type condition `{type_condition}` is not assignable to this type"),
                );
            }
            DiagnosticData::DisallowedVariableUsage {
                variable,
                variable_type,
                variable_location,
                ..
            } => {
                report.with_label_opt(
                    *variable_location,
                    format_args!(
                        "variable `${variable}` of type `{variable_type}` is declared here"
                    ),
                );
                report.with_label_opt(
                    self.location,
                    format_args!("variable `${variable}` used here"),
                );
            }
            DiagnosticData::RecursionError {} => {}
        }
    }
}

fn label_recursive_trace<T>(
    report: &mut CliReport,
    trace: &[Node<T>],
    original_name: &str,
    get_name: impl Fn(&T) -> &str,
) {
    if let Some((cyclical_application, path)) = trace.split_first() {
        let mut prev_name = original_name;
        for node in path.iter().rev() {
            let name = get_name(node);
            report.with_label_opt(
                node.location(),
                format!("`{prev_name}` references `{name}` here..."),
            );
            prev_name = name;
        }

        report.with_label_opt(
            cyclical_application.location(),
            format!("`{prev_name}` circularly references `{original_name}` here"),
        );
    }
}

struct CommaSeparated<'a, It>(&'a It);
impl<'a, T, It> fmt::Display for CommaSeparated<'a, It>
where
    T: fmt::Display,
    &'a It: IntoIterator<Item = T>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut it = self.0.into_iter();
        if let Some(element) = it.next() {
            element.fmt(f)?;
        }
        for element in it {
            f.write_str(", ")?;
            element.fmt(f)?;
        }
        Ok(())
    }
}

/// Formatter that describes a name, or describes an anonymous element if there is no name.
pub(crate) struct NameOrAnon<'a, T> {
    pub name: Option<&'a T>,
    pub if_some_prefix: &'a str,
    pub if_none: &'a str,
}
impl<T> fmt::Display for NameOrAnon<'_, T>
where
    T: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.name {
            Some(name) => write!(f, "{} `{}`", self.if_some_prefix, name),
            None => f.write_str(self.if_none),
        }
    }
}

fn fragment_name_or_inline<T>(name: &'_ Option<T>) -> NameOrAnon<'_, T> {
    NameOrAnon {
        name: name.as_ref(),
        if_some_prefix: "fragment",
        if_none: "inline fragment",
    }
}
