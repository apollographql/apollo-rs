//! A parser for the GraphQL query language.

mod lexer;
mod parser;

mod error;

pub use lexer::*;
pub use parser::*;
pub use error::Error;

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