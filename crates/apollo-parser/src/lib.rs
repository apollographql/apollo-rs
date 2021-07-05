//! A parser for the GraphQL query language.

pub mod lexer;
pub mod parser;

mod token_kind;
mod error;

pub use lexer::*;
pub use parser::*;
pub use error::Error;

#[macro_export]
macro_rules! bail {
    ($data:expr, $($tt:tt)*) => {
        return Err($crate::lexer::Error::new(
            format!($($tt)*),
            $data.to_string(),
        ))
    };
}

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