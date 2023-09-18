//! Example usage:
//!
//! ```
#![doc = include_str!("../tests/doc_example.rs")]
//! ```

#[macro_use]
mod resolver;
mod execution;
mod input_coercion;
mod response;
mod result_coercion;
mod schema_introspection;

pub use self::execution::get_operation;
pub use self::input_coercion::VariableValues;
pub use self::response::Error;
pub use self::response::Location;
pub use self::response::PathElement;
pub use self::response::RequestErrorResponse;
pub use self::response::Response;
pub use self::response::EXTENSION_VALIDATION_SHOULD_HAVE_CAUGHT_THIS;
pub use self::schema_introspection::SchemaIntrospectionQuery;
pub use serde_json_bytes::ByteString;
pub use serde_json_bytes::Value as JsonValue;

/// Represents a JSON object
pub type JsonMap = serde_json_bytes::Map<ByteString, JsonValue>;
