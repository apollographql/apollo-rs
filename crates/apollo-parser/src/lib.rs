//! A parser for the GraphQL query language.
//!
//! ## Example
//! ```rust
//! use apollo_parser::Parser;
//! use apollo_parser::ast::{Definition, ObjectTypeDefinition};
//!
//! let input = "
//! type ProductDimension {
//!   size: String
//!   weight: Float @tag(name: \"hi from inventory value type field\")
//! }
//! ";
//! let parser = Parser::new(input);
//! let ast = parser.parse();
//! assert!(ast.errors().is_empty());
//!
//! let doc = ast.document();
//!
//! for def in doc.definitions() {
//!     if let Definition::ObjectTypeDefinition(object_type) = def {
//!         assert_eq!(object_type.name().unwrap().text(), "ProductDimension");
//!         for field_def in object_type.fields_definition().unwrap().field_definitions() {
//!             println!("{}", field_def.name().unwrap().text()); // size weight
//!         }
//!     }
//! }
//! ```

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
        Err($crate::error::Error::new(
            format!($($tt)*),
            $data.to_string(),
        ))
    };
}

#[macro_export]
macro_rules! create_err {
    ($data:expr, $($tt:tt)*) => {
        $crate::error::Error::new(
            format!($($tt)*),
            $data.to_string(),
        )
    };
}

/// Return early with an error.
#[macro_export]
macro_rules! bail {
    ($data:expr, $($tt:tt)*) => {
        return Err($crate::error::Error::new(
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
