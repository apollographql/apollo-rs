mod error_formatting;
mod executable;
mod extensions;
mod field_set;
mod field_type;
mod introspection;
mod introspection_max_depth;
mod locations;
mod merge_schemas;
/// Formerly in src/lib.rs
mod misc;
mod name;
mod parser;
mod schema;
mod serde;
mod validation;

#[path = "../examples/rename.rs"]
mod rename;
