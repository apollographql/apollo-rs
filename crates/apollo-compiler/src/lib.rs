#![doc = include_str!("../README.md")]
#![deny(unreachable_pub)]
// Name contains an AtomicU32 that lazily caches its interned
// symbol. The cached value is a deterministic function of the string, so
// Eq/Hash outcomes never change and Names are sound map keys.
#![allow(clippy::mutable_key_type)]

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
mod symbol;
pub mod validation;

pub use self::executable::ExecutableDocument;
pub use self::name::InvalidNameError;
pub use self::name::Name;
pub use self::node::ExtensionId;

/// Freezes the global name-interning table.
///
/// [`Name`]s intern their string in a process-global table: equal interned
/// names compare and hash by a small integer symbol instead of their string
/// bytes. After freezing, names whose strings are not already interned fall
/// back to string-based equality and hashing (both are consistent: with a
/// frozen table, whether a string is interned is a pure function of the
/// string) and can no longer grow the table.
///
/// Servers should call this once after building their schema(s) and before
/// parsing untrusted executable documents. This bounds the table's size by
/// the schemas' contents: hostile documents full of unique names cannot
/// grow it. The call must happen-before any untrusted parsing begins.
///
/// Later schema changes degrade gracefully: names not in the frozen table
/// use string-based semantics, at the same cost as before interning
/// existed.
pub fn freeze_interning() {
    self::symbol::freeze()
}
pub use self::node::Node;
pub use self::schema::Schema;
