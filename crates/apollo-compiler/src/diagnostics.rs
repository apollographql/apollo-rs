use miette::{Diagnostic, SourceSpan};
use thiserror::Error;

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum ApolloDiagnostic {
    MissingIdent(MissingIdent),
    UniqueDefinition(UniqueDefinition),
    SingleRootField(SingleRootField),
    UnsupportedOperation(UnsupportedOperation),
}

#[derive(Diagnostic, Debug, Error, Clone, Hash, PartialEq, Eq)]
#[error("expected identifier")]
#[diagnostic(code("apollo-compiler validation error."))]
pub struct MissingIdent {
    #[source_code]
    pub src: String,

    #[label = "provide a name for this definition"]
    pub definition: SourceSpan,

    #[help]
    pub help: Option<String>,
}

#[derive(Diagnostic, Debug, Error, Clone, Hash, PartialEq, Eq)]
#[error("the name `{}` is defined multiple times in the document", self.ty)]
#[diagnostic(code("apollo-compiler validation error."))]
pub struct UniqueDefinition {
    // current definition
    pub ty: String,

    #[source_code]
    pub src: String,

    #[label("previous definition of `{}` here", self.ty)]
    pub original_definition: SourceSpan,

    #[label("`{}` is redefined here", self.ty)]
    pub redefined_definition: SourceSpan,

    #[help]
    pub help: Option<String>,
}

#[derive(Diagnostic, Debug, Error, Clone, Hash, PartialEq, Eq)]
#[error("Subscriptions operations can only have one root field")]
#[diagnostic(code("apollo-compiler validation error."))]
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
#[diagnostic(code("apollo-compiler validation error."))]
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

// // NOTE @lrlna: only syntax errors currently have the source data.
// //
// // TODO: figure out a nice way of going back to the AST and get its source data
// // given a current Value, which will make sure the rest of the diagnostics have
// // source data.
// #[derive(Clone, Debug, Hash, PartialEq, Eq)]
// pub enum ApolloDiagnostic {
//     Error(ErrorDiagnostic),
//     Warning(WarningDiagnostic),
//     Hint(HintDiagnostic),
//     Suggestion(SuggestionDiagnostic),
// }
//
// #[derive(Clone, Debug, Hash, PartialEq, Eq)]
// pub enum ErrorDiagnostic {
//     BuiltInScalarDefinition {
//         message: String,
//         scalar: String,
//     },
//     MissingField {
//         message: String,
//         field: String,
//         current_definition: String,
//         super_definition: String,
//     },
//     QueryRootOperationType(String),
//     RecursiveDefinition {
//         message: String,
//         definition: String,
//     },
//     ScalarSpecificationURL {
//         message: String,
//         scalar: String,
//     },
//     SyntaxError {
//         message: String,
//         data: String,
//         index: usize,
//     },
//     TransitiveImplementedInterfaces {
//         message: String,
//         interface: String,
//         missing_implemented_interface: String,
//     },
//     UniqueRootOperationType {
//         message: String,
//         named_type: String,
//         operation_type: String,
//     },
//     UniqueValue {
//         message: String,
//         value: String,
//     },
//     UniqueField {
//         message: String,
//     },
//     UndefinedDefinition {
//         message: String,
//         missing_definition: String,
//     },
//     UndefinedVariable {
//         message: String,
//         variable: String,
//     },
// }
//
// #[derive(Clone, Debug, Hash, PartialEq, Eq)]
// pub enum WarningDiagnostic {
//     UnusedVariable { message: String, variable: String },
//     CapitalizedValue { message: String, value: String },
// }
//
// #[derive(Clone, Debug, Hash, PartialEq, Eq)]
// pub enum HintDiagnostic {}
//
// #[derive(Clone, Debug, Hash, PartialEq, Eq)]
// pub enum SuggestionDiagnostic {}
