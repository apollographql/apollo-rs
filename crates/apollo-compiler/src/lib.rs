#![doc = include_str!("../README.md")]

#[macro_use]
mod macros;
pub mod ast;
pub mod coordinate;
mod database;
pub mod diagnostic;
pub mod executable;
pub mod execution;
mod node;
mod node_str;
mod parser;
pub mod schema;
pub mod validation;

use crate::database::{InputDatabase, ReprDatabase, Source};
use crate::validation::ValidationDatabase;

pub use self::executable::ExecutableDocument;
pub use self::node::Node;
pub use self::node_str::NodeStr;
pub use self::parser::{parse_mixed_validate, Parser, SourceFile, SourceMap};
pub use self::schema::Schema;
