#![doc = include_str!("../README.md")]
#![deny(unreachable_pub)]

#[macro_use]
mod macros;
pub mod ast;
pub mod collections;
pub mod coordinate;
pub mod diagnostic;
pub mod executable;
mod intern;
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
pub use self::node::ExtensionId;
pub use self::node::Node;
pub use self::schema::Schema;

/// Freezes the global name-interning table.
///
/// [`Name`]s intern their string in a process-global table: equal interned
/// names share one copy of the string and compare and hash by its address
/// instead of its bytes. After freezing, names whose strings are not already
/// interned fall back to string-based equality and hashing (both are
/// consistent: with a frozen table, whether a string is interned is a pure
/// function of the string) and can no longer grow the table.
///
/// Servers should call this once after building their schema(s) and before
/// parsing untrusted executable documents. This bounds the table's size by
/// the schemas' contents: hostile documents full of unique names cannot
/// grow it.
///
/// Later schema changes degrade gracefully: names not in the frozen table
/// use string-based semantics, at the same cost as before interning
/// existed.
pub fn freeze_interning() {
    self::intern::freeze()
}
