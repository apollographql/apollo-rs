use std::fmt;

use miette::{Diagnostic, Report, SourceSpan};
use thiserror::Error;

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum ApolloDiagnostic {
    MissingIdent(MissingIdent),
    MissingField(MissingField),
    UniqueDefinition(UniqueDefinition),
    SingleRootField(SingleRootField),
    UnsupportedOperation(UnsupportedOperation),
    SyntaxError(SyntaxError),
    UniqueField(UniqueField),
    UndefinedDefinition(UndefinedDefinition),
    UndefinedField(UndefinedField),
    UniqueArgument(UniqueArgument),
    RecursiveDefinition(RecursiveDefinition),
    TransitiveImplementedInterfaces(TransitiveImplementedInterfaces),
    QueryRootOperationType(QueryRootOperationType),
    BuiltInScalarDefinition(BuiltInScalarDefinition),
    ScalarSpecificationURL(ScalarSpecificationURL),
    CapitalizedValue(CapitalizedValue),
    UnusedVariable(UnusedVariable),
    OutputType(OutputType),
    ObjectType(ObjectType),
}

impl ApolloDiagnostic {
    pub fn is_error(&self) -> bool {
        matches!(
            self,
            ApolloDiagnostic::MissingIdent(_)
                | ApolloDiagnostic::MissingField(_)
                | ApolloDiagnostic::UniqueDefinition(_)
                | ApolloDiagnostic::SingleRootField(_)
                | ApolloDiagnostic::UnsupportedOperation(_)
                | ApolloDiagnostic::SyntaxError(_)
                | ApolloDiagnostic::UniqueField(_)
                | ApolloDiagnostic::UndefinedDefinition(_)
                | ApolloDiagnostic::RecursiveDefinition(_)
                | ApolloDiagnostic::TransitiveImplementedInterfaces(_)
                | ApolloDiagnostic::QueryRootOperationType(_)
                | ApolloDiagnostic::UndefinedField(_)
                | ApolloDiagnostic::UniqueArgument(_)
                | ApolloDiagnostic::BuiltInScalarDefinition(_)
                | ApolloDiagnostic::OutputType(_)
                | ApolloDiagnostic::ObjectType(_)
        )
    }

    pub fn is_warning(&self) -> bool {
        matches!(
            self,
            ApolloDiagnostic::CapitalizedValue(_) | ApolloDiagnostic::UnusedVariable(_)
        )
    }

    pub fn is_advice(&self) -> bool {
        matches!(self, ApolloDiagnostic::ScalarSpecificationURL(_))
    }

    pub fn report(&self) -> Report {
        match self {
            ApolloDiagnostic::MissingIdent(diagnostic) => Report::new(diagnostic.clone()),
            ApolloDiagnostic::UniqueDefinition(diagnostic) => Report::new(diagnostic.clone()),
            ApolloDiagnostic::SingleRootField(diagnostic) => Report::new(diagnostic.clone()),
            ApolloDiagnostic::UnsupportedOperation(diagnostic) => Report::new(diagnostic.clone()),
            ApolloDiagnostic::SyntaxError(diagnostic) => Report::new(diagnostic.clone()),
            ApolloDiagnostic::UniqueField(diagnostic) => Report::new(diagnostic.clone()),
            ApolloDiagnostic::RecursiveDefinition(diagnostic) => Report::new(diagnostic.clone()),
            ApolloDiagnostic::UndefinedDefinition(diagnostic) => Report::new(diagnostic.clone()),
            ApolloDiagnostic::TransitiveImplementedInterfaces(diagnostic) => {
                Report::new(diagnostic.clone())
            }
            ApolloDiagnostic::QueryRootOperationType(diagnostic) => Report::new(diagnostic.clone()),
            ApolloDiagnostic::BuiltInScalarDefinition(diagnostic) => {
                Report::new(diagnostic.clone())
            }
            ApolloDiagnostic::ScalarSpecificationURL(diagnostic) => Report::new(diagnostic.clone()),
            ApolloDiagnostic::CapitalizedValue(diagnostic) => Report::new(diagnostic.clone()),
            ApolloDiagnostic::UnusedVariable(diagnostic) => Report::new(diagnostic.clone()),
            ApolloDiagnostic::MissingField(diagnostic) => Report::new(diagnostic.clone()),
            ApolloDiagnostic::OutputType(diagnostic) => Report::new(diagnostic.clone()),
            ApolloDiagnostic::ObjectType(diagnostic) => Report::new(diagnostic.clone()),
            ApolloDiagnostic::UndefinedField(diagnostic) => Report::new(diagnostic.clone()),
            ApolloDiagnostic::UniqueArgument(diagnostic) => Report::new(diagnostic.clone()),
        }
    }
}

impl fmt::Display for ApolloDiagnostic {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "{:?}", self.report())
    }
}

#[derive(Diagnostic, Debug, Error, Clone, Hash, PartialEq, Eq)]
#[error("expected identifier")]
#[diagnostic(code("apollo-compiler validation error"))]
pub struct MissingIdent {
    #[source_code]
    pub src: String,

    #[label = "provide a name for this definition"]
    pub definition: SourceSpan,

    #[help]
    pub help: Option<String>,
}

#[derive(Diagnostic, Debug, Error, Clone, Hash, PartialEq, Eq)]
#[error("missing `{}` field", self.ty)]
#[diagnostic(code("apollo-compiler validation error"))]
pub struct MissingField {
    // current field that should be defined
    pub ty: String,

    #[source_code]
    pub src: String,

    #[label("`{}` was originally defined here", self.ty)]
    pub super_definition: SourceSpan,

    #[label("add `{}` field to this interface", self.ty)]
    pub current_definition: SourceSpan,

    #[help]
    pub help: Option<String>,
}

#[derive(Diagnostic, Debug, Error, Clone, Hash, PartialEq, Eq)]
#[error("the {} `{}` is defined multiple times in the document", self.ty, self.name)]
#[diagnostic(code("apollo-compiler validation error"))]
pub struct UniqueDefinition {
    // current definition
    pub name: String,

    // current definition type
    pub ty: String,

    #[source_code]
    pub src: String,

    #[label("previous definition of `{}` here", self.name)]
    pub original_definition: SourceSpan,

    #[label("`{}` is redefined here", self.name)]
    pub redefined_definition: SourceSpan,

    #[help]
    pub help: Option<String>,
}

#[derive(Diagnostic, Debug, Error, Clone, Hash, PartialEq, Eq)]
#[error("Subscriptions operations can only have one root field")]
#[diagnostic(code("apollo-compiler validation error"))]
pub struct SingleRootField {
    #[source_code]
    pub src: String,

    pub fields: usize,

    #[label("subscription with {} root fields", self.fields)]
    pub subscription: SourceSpan,

    #[help]
    pub help: Option<String>,
}

#[derive(Diagnostic, Debug, Error, Clone, Hash, PartialEq, Eq)]
#[error("{} root operation type is not defined", self.ty)]
#[diagnostic(code("apollo-compiler validation error"))]
pub struct UnsupportedOperation {
    // current operation type: subscription, mutation, query
    pub ty: String,

    #[source_code]
    pub src: String,

    #[label("{} operation is not defined in the schema and is therefore not supported", self.ty)]
    pub operation: SourceSpan,

    #[label("consider defining a {} root operation type here", self.ty)]
    pub schema: Option<SourceSpan>,

    #[help]
    pub help: Option<String>,
}

#[derive(Diagnostic, Debug, Error, Clone, Hash, PartialEq, Eq)]
#[error("root operation type is not defined")]
#[diagnostic(code("apollo-compiler syntax error"))]
pub struct SyntaxError {
    pub message: String,

    #[source_code]
    pub src: String,

    #[label("{}", self.message)]
    pub span: SourceSpan,
}

#[derive(Diagnostic, Debug, Error, Clone, Hash, PartialEq, Eq)]
#[error("Fields must be unique in a definition")]
#[diagnostic(code("apollo-compiler validation error"))]
pub struct UniqueField {
    // current operation type: subscription, mutation, query
    pub field: String,

    #[source_code]
    pub src: String,

    #[label("previous definition of `{}` field here", self.field)]
    pub original_field: SourceSpan,

    #[label("`{}` is redefined here", self.field)]
    pub redefined_field: SourceSpan,

    #[help]
    pub help: Option<String>,
}

#[derive(Diagnostic, Debug, Error, Clone, Hash, PartialEq, Eq)]
#[error("{}", self.message)]
#[diagnostic(code("apollo-compiler validation error"))]
pub struct RecursiveDefinition {
    #[source_code]
    pub src: String,

    #[label("{}", self.definition_label)]
    pub definition: SourceSpan,

    pub definition_label: String,

    pub message: String,
}

#[derive(Diagnostic, Debug, Error, Clone, Hash, PartialEq, Eq)]
#[error("cannot find type `{}` in this document", self.ty)]
#[diagnostic(code("apollo-compiler validation error"))]
pub struct UndefinedDefinition {
    // current type not in scope
    pub ty: String,

    #[source_code]
    pub src: String,

    #[label("not found in this scope")]
    pub definition: SourceSpan,
}

#[derive(Diagnostic, Debug, Error, Clone, Hash, PartialEq, Eq)]
#[error("Transitively implemented interfaces must also be defined on an implementing interface")]
#[diagnostic(code("apollo-compiler validation error"))]
pub struct TransitiveImplementedInterfaces {
    // interface that should be defined
    pub missing_interface: String,

    #[source_code]
    pub src: String,

    #[label("{} must also be implemented here", self.missing_interface)]
    pub definition: SourceSpan,
}

#[derive(Diagnostic, Debug, Error, Clone, Hash, PartialEq, Eq)]
#[error("Missing query root operation type in schema definition")]
#[diagnostic(code("apollo-compiler validation error"))]
pub struct QueryRootOperationType {
    #[source_code]
    pub src: String,

    #[label("`query` root operation type must be defined here")]
    pub schema: SourceSpan,
}

#[derive(Diagnostic, Debug, Error, Clone, Hash, PartialEq, Eq)]
#[error("Built-in scalars must be omitted for brevity")]
#[diagnostic(code("apollo-compiler validation error"))]
pub struct BuiltInScalarDefinition {
    #[source_code]
    pub src: String,

    #[label("remove this scalar definition")]
    pub scalar: SourceSpan,
}

#[derive(Diagnostic, Debug, Error, Clone, Hash, PartialEq, Eq)]
#[error("Custom scalars should provide a scalar specification URL via the @specifiedBy directive")]
#[diagnostic(code("apollo-compiler validation advice"), severity(advice))]
pub struct ScalarSpecificationURL {
    #[source_code]
    pub src: String,

    #[label("consider adding a @specifiedBy directive to this scalar definition")]
    pub scalar: SourceSpan,
}

#[derive(Diagnostic, Debug, Error, Clone, Hash, PartialEq, Eq)]
#[error("values in an Enum Definition should be capitalized")]
#[diagnostic(code("apollo-compiler validation warning"), severity(warning))]
pub struct CapitalizedValue {
    pub ty: String,

    #[source_code]
    pub src: String,

    #[label("consider capitalizing {}", self.ty)]
    pub value: SourceSpan,
}

#[derive(Diagnostic, Debug, Error, Clone, Hash, PartialEq, Eq)]
#[error("unused variable: `{}`", self.ty)]
#[diagnostic(code("apollo-compiler validation error"))]
pub struct UnusedVariable {
    pub ty: String,

    #[source_code]
    pub src: String,

    #[label("unused variable")]
    pub definition: SourceSpan,
}

#[derive(Diagnostic, Debug, Error, Clone, Hash, PartialEq, Eq)]
#[error("`{}` field must return an output type", self.name)]
#[diagnostic(
    code("apollo-compiler validation error"),
    help("Scalars, Objects, Interfaces, Unions and Enums are output types. Change `{}` field to return one of these output types.", self.name)
)]
pub struct OutputType {
    // field name
    pub name: String,
    // field type
    pub ty: String,

    #[source_code]
    pub src: String,

    #[label("this is of `{}` type", self.ty)]
    pub definition: SourceSpan,
}

#[derive(Diagnostic, Debug, Error, Clone, Hash, PartialEq, Eq)]
#[error("`{}` field must return an output type", self.name)]
#[diagnostic(
    code("apollo-compiler validation error"),
    help("Union members must be of base Object Type. `{}` is of `{}` type", self.name, self.ty)
)]
pub struct ObjectType {
    // union member
    pub name: String,
    // actual type
    pub ty: String,

    #[source_code]
    pub src: String,

    #[label("this is of `{}` type", self.ty)]
    pub definition: SourceSpan,
}

#[derive(Diagnostic, Debug, Error, Clone, Hash, PartialEq, Eq)]
#[error("Cannot query `{}` field", self.field)]
/// Returns `true` if the definition is either a [`ScalarTypeDefinition`],
#[diagnostic(code("apollo-compiler validation error"))]
pub struct UndefinedField {
    // field name
    pub field: String,

    #[source_code]
    pub src: String,

    #[label("`{}` field is not in scope", self.field)]
    pub definition: SourceSpan,

    #[help]
    pub help: String,
}

#[derive(Diagnostic, Debug, Error, Clone, Hash, PartialEq, Eq)]
#[error("the argument `{}` is defined multiple times", self.name)]
#[diagnostic(code("apollo-compiler validation error"))]
pub struct UniqueArgument {
    // current definition
    pub name: String,

    #[source_code]
    pub src: String,

    #[label("previous definition of `{}` here", self.name)]
    pub original_definition: SourceSpan,

    #[label("`{}` is redefined here", self.name)]
    pub redefined_definition: SourceSpan,

    #[help]
    pub help: Option<String>,
}
