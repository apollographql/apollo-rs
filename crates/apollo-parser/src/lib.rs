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
pub use crate::parser::{Parser, SyntaxElement, SyntaxKind, SyntaxNode, SyntaxTree};
pub(crate) use crate::parser::{SyntaxNodeChildren, SyntaxToken, TokenText};
pub use rowan::TextRange;

pub use crate::error::Error;
pub use crate::limit::LimitTracker;
