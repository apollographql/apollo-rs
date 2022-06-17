use miette::{Diagnostic, SourceSpan};
use thiserror::Error;

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum ApolloDiagnostic {
    MissingIdent(MissingIdent),
}

#[derive(Diagnostic, Debug, Error, Clone, Hash, PartialEq, Eq)]
#[error("expected identifier")]
pub struct MissingIdent {
    // This will underline/mark the specific code inside the larger
    // snippet context.
    #[label = "expected identifier"]
    pub err_span: SourceSpan,

    #[help]
    pub hint: Option<String>,

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
//     MissingIdent(String),
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
//     SingleRootField(String),
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
//     UniqueDefinition {
//         message: String,
//         definition: String,
//     },
//     UnsupportedOperation {
//         message: String,
//         operation: Option<String>,
//     },
//     UniqueOperationDefinition {
//         message: String,
//         operation: String,
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
