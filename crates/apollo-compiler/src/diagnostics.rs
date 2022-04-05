// NOTE @lrlna: only syntax errors currently have the source data.
//
// TODO: figure out a nice way of going back to the AST and get its source data
// given a current Value, which will make sure the rest of the diagnostics have
// source data.
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum ApolloDiagnostic {
    MissingIdent(String),
    SingleRootField(String),
    UniqueOperationDefinition {
        message: String,
        operation: String,
    },
    UndefinedVariablesError {
        message: String,
        variable: String,
    },
    UnusedVariablesWarning {
        message: String,
        variable: String,
    },
    SyntaxError {
        message: String,
        data: String,
        index: usize,
    },
}
