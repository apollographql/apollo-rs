#![doc = include_str!("../README.md")]
#![allow(deprecated)] // TODO: after the transition, remove this and `ast::Name`

#[macro_use]
mod macros;
pub mod ast;
pub mod coordinate;
pub mod diagnostic;
pub mod executable;
pub mod execution;
mod name;
mod node;
mod parser;
pub mod schema;
pub mod validation;

pub use self::executable::ExecutableDocument;
pub use self::name::{InvalidNameError, Name, StaticOrArc};
pub use self::node::{FileId, Node, NodeLocation};
pub use self::parser::{parse_mixed_validate, Parser, SourceFile, SourceMap};
pub use self::schema::Schema;
