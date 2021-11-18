#![doc = include_str!("../README.md")]

mod lexer;
#[cfg(test)]
mod tests;

pub mod ast;
mod error;
mod parser;

pub use crate::lexer::{Lexer, LexerIterator};
pub use crate::lexer::{Token, TokenKind};
pub(crate) use crate::parser::{
    SyntaxElement, SyntaxKind, SyntaxNodeChildren, SyntaxToken, TokenText,
};

pub use crate::error::Error;
pub use crate::parser::{LimitTracker, Parser, SyntaxNode, SyntaxTree};
