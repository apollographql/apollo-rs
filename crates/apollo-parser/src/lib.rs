#![doc = include_str!("../README.md")]

mod lexer;
#[cfg(test)]
mod tests;

pub mod ast;
mod error;
mod parser;

#[cfg(test)]
pub(crate) use crate::lexer::Lexer;
pub(crate) use crate::lexer::{Token, TokenKind};
pub(crate) use crate::parser::{
    SyntaxElement, SyntaxKind, SyntaxNode, SyntaxNodeChildren, SyntaxToken, TokenText,
};

pub use crate::error::Error;
pub use crate::parser::{Parser, SyntaxTree};
