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
pub use self::name::InvalidNameError;
pub use self::name::Name;
pub use self::node::FileId;
pub use self::node::Node;
pub use self::node::NodeLocation;
pub use self::parser::parse_mixed_validate;
pub use self::parser::Parser;
pub use self::parser::SourceFile;
pub use self::parser::SourceMap;
pub use self::schema::Schema;
