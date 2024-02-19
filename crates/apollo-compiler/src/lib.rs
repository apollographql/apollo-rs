#![doc = include_str!("../README.md")]

#[macro_use]
mod macros;
pub mod ast;
pub mod coordinate;
pub mod diagnostic;
pub mod executable;
pub mod execution;
mod node;
mod node_str;
mod parser;
pub mod schema;
pub mod validation;

pub use self::executable::ExecutableDocument;
pub use self::node::{FileId, Node, NodeLocation};
pub use self::node_str::NodeStr;
pub use self::parser::{parse_mixed_validate, Parser, SourceFile, SourceMap};
pub use self::schema::Schema;
