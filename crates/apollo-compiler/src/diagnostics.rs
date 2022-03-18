use miette::{Diagnostic, NamedSource, SourceSpan};
use thiserror::Error;

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum ApolloDiagnostic {
    MissingIdent(String),
    UndefinedInterfacesError,
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

#[derive(Error, Debug, Diagnostic)]
#[error("cannot find `{}` interface in this scope", self.ty)]
#[diagnostic(code("apollo-parser: semantic error"))]
struct UndefinedInterfacesError {
    ty: String,
    #[source_code]
    src: NamedSource,
    message: String,
    #[label("{}", self.message)]
    span: SourceSpan,
}

// #[derive(Error, Debug, Diagnostic)]
// #[error("cannot find `{}` variable in this scope", self.ty)]
// #[diagnostic(code("apollo-parser: semantic error"))]
// struct UndefinedVariablesError {
//     ty: String,
//     #[source_code]
//     src: NamedSource,
//     message: String,
//     #[label("{}", self.message)]
//     span: SourceSpan,
// }
