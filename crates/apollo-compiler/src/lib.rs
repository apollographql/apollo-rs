#![doc = include_str!("../README.md")]
#![deny(unreachable_pub)]

#[macro_use]
mod macros;
pub mod ast;
pub mod collections;
pub mod coordinate;
pub mod diagnostic;
pub mod executable;
pub mod introspection;
mod name;
mod node;
pub mod parser;
pub mod request;
pub mod resolvers;
pub mod response;
pub mod schema;
pub mod validation;

pub use self::executable::ExecutableDocument;
pub use self::name::InvalidNameError;
pub use self::name::Name;
pub use self::node::Node;
pub use self::schema::Schema;
