//! A parser for the GraphQL query language.
//!
//! ## Examples
//!
//! ### An example to get field names:
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
//!
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
//!
//! ### An example to get variables used in a query:
//! ```rust
//! use apollo_parser::{Parser};
//! use apollo_parser::ast::{Definition, OperationDefinition};
//!
//! let input = "
//! query GraphQuery($graph_id: ID!, $variant: String) {
//!   service(id: $graph_id) {
//!     schema(tag: $variant) {
//!       document
//!     }
//!   }
//! }
//! ";
//!
//! let parser = Parser::new(input);
//! let ast = parser.parse();
//! assert!(&ast.errors().is_empty());
//!
//! let doc = ast.document();
//!
//! for def in doc.definitions() {
//!     if let Definition::OperationDefinition(op_def) = def {
//!         assert_eq!(op_def.name().unwrap().text(), "GraphQuery");
//!
//!         let variable_defs = op_def.variable_definitions();
//!         let variables: Vec<String> = variable_defs
//!             .iter()
//!             .map(|v| v.variable_definitions())
//!             .flatten()
//!             .filter_map(|v| Some(v.variable()?.name()?.text().to_string()))
//!             .collect();
//!         assert_eq!(
//!             variables.as_slice(),
//!             ["graph_id".to_string(), "variant".to_string()]
//!         );
//!     }
//! }
//! ```
mod lexer;
#[cfg(test)]
mod tests;

pub mod ast;
pub mod error;
pub mod parser;

#[cfg(test)]
pub(crate) use crate::lexer::Lexer;
pub(crate) use crate::lexer::{Location, Token, TokenKind};
pub(crate) use crate::parser::{
    SyntaxElement, SyntaxKind, SyntaxNode, SyntaxNodeChildren, SyntaxToken, TokenText,
};

pub use crate::error::Error;
pub use crate::parser::{Parser, SyntaxTree};

#[macro_export]
macro_rules! create_err {
    ($data:expr, $($tt:tt)*) => {
        $crate::error::Error::new(
            format!($($tt)*),
            $data.to_string(),
        )
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
