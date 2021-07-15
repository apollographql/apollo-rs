//! A parser for the GraphQL query language.

pub mod ast;
mod error;
mod lexer;
mod parser;

pub use error::Error;
pub use lexer::*;
pub use parser::*;

#[macro_export]
macro_rules! format_err {
    ($data:expr, $($tt:tt)*) => {
        Err($crate::lexer::Error::new(
            format!($($tt)*),
            $data.to_string(),
        ))
    };
}

/// Return early with an error.
#[macro_export]
macro_rules! bail {
    ($data:expr, $($tt:tt)*) => {
        return Err($crate::lexer::Error::new(
            format!($($tt)*),
            $data.to_string(),
        ))
    };
}

/// Return early with an error if a condition is not satisfied.
#[macro_export]
macro_rules! ensure {
    ($cond:expr, $data:expr, $($tt:tt)*) => {
        if !$cond {
            return Err($crate::lexer::Error::new(
                format!($($tt)*),
                $data.to_string(),
            ))
        }
    };
}
