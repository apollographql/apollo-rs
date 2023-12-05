//! APIs related to [executing a GraphQL request][execution]
//! and returning a [GraphQL response][response]
//!
//! [execution]: https://spec.graphql.org/October2021/#sec-Execution
//! [response]: https://spec.graphql.org/October2021/#sec-Response

pub(crate) mod engine;
mod input_coercion;
#[macro_use]
pub(crate) mod resolver;
mod introspection;
mod response;
mod result_coercion;

pub use self::input_coercion::coerce_variable_values;
pub use self::input_coercion::InputCoercionError;
pub use self::introspection::SchemaIntrospection;
pub use self::response::GraphQLError;
pub use self::response::GraphQLLocation;
pub use self::response::PathElement;
pub use self::response::Response;
pub use self::response::ResponseData;
/// Re-export of the version of the `serde_json_bytes` crate used for [`JsonValue`] and [`JsonMap`]
pub use serde_json_bytes;

/// A JSON-compatible dynamically-typed value.
///
/// Note: [`serde_json_bytes::Value`] is similar
/// to [`serde_json::Value`][serde_json_bytes::serde_json::Value]
/// but uses its reference-counted [`ByteString`][serde_json_bytes::ByteString]
/// for string values and map keys.
pub type JsonValue = serde_json_bytes::Value;

/// A JSON-compatible object/map with string keys and dynamically-typed values.
pub type JsonMap = serde_json_bytes::Map<serde_json_bytes::ByteString, JsonValue>;
