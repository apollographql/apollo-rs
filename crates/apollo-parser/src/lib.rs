#![doc = include_str!("../README.md")]

mod lexer;
#[cfg(test)]
mod tests;

pub mod ast;
mod bowstring;
mod error;
mod limit;
pub mod mir;
mod parser;

pub use crate::bowstring::BowString;
pub use crate::lexer::Lexer;
pub use crate::lexer::{Token, TokenKind};
pub(crate) use crate::parser::{
    SyntaxElement, SyntaxKind, SyntaxNodeChildren, SyntaxToken, TokenText,
};

pub use crate::error::Error;
pub use crate::limit::LimitTracker;
pub use crate::parser::{Parser, SyntaxNode, SyntaxTree};
