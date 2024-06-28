#![doc = include_str!("../README.md")]

mod lexer;
#[cfg(test)]
mod tests;

pub mod cst;
mod error;
mod limit;
mod parser;

pub use crate::error::Error;
pub use crate::lexer::Lexer;
pub use crate::lexer::Token;
pub use crate::lexer::TokenKind;
pub use crate::limit::LimitTracker;
pub use crate::parser::Parser;
pub use crate::parser::SyntaxElement;
pub use crate::parser::SyntaxKind;
pub use crate::parser::SyntaxNode;
pub(crate) use crate::parser::SyntaxNodeChildren;
pub(crate) use crate::parser::SyntaxToken;
pub use crate::parser::SyntaxTree;
pub(crate) use crate::parser::TokenText;
pub use rowan::TextRange;
