#![doc = include_str!("../README.md")]

mod lexer;
#[cfg(test)]
mod tests;

pub mod cst;
mod error;
mod limit;
mod parser;

pub use crate::lexer::Lexer;
pub use crate::lexer::{Token, TokenKind};
pub use crate::parser::SyntaxKind;
pub(crate) use crate::parser::{SyntaxElement, SyntaxNodeChildren, SyntaxToken, TokenText};

pub use crate::error::Error;
pub use crate::limit::LimitTracker;
pub use crate::parser::{Parser, SyntaxNode, SyntaxTree};
