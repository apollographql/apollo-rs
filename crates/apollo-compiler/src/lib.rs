#![doc = include_str!("../README.md")]

#[macro_use]
mod macros;
pub mod ast;
pub mod collections;
pub mod coordinate;
pub mod diagnostic;
pub mod executable;
pub mod execution;
mod name;
mod node;
pub mod parser;
pub mod schema;
pub mod validation;

pub use self::executable::ExecutableDocument;
pub use self::name::InvalidNameError;
pub use self::name::Name;
pub use self::node::FileId;
pub use self::node::Node;
pub use self::node::NodeLocation;
pub use self::parser::parse_mixed_validate;
pub use self::schema::Schema;
