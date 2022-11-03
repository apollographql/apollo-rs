#![doc = include_str!("../README.md")]

mod lexer;
#[cfg(test)]
mod tests;

pub mod ast;
mod error;
mod parser;
mod limit;

pub use crate::lexer::Lexer;
pub use crate::lexer::{Token, TokenKind};
pub(crate) use crate::parser::{
    SyntaxElement, SyntaxKind, SyntaxNodeChildren, SyntaxToken, TokenText,
};

pub use crate::error::Error;
pub use crate::parser::{Parser, SyntaxNode, SyntaxTree};
pub use crate::limit::LimitTracker;
