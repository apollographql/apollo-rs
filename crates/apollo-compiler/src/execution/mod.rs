//! APIs related to [executing a GraphQL request][execution]
//! and returning a [GraphQL response][response]
//!
//! [execution]: https://spec.graphql.org/October2021/#sec-Execution
//! [response]: https://spec.graphql.org/October2021/#sec-Response

#[macro_use]
mod resolver;
mod engine;
mod input_coercion;
mod introspection_execute;
mod introspection_max_depth;
mod introspection_split;
mod response;
mod result_coercion;

pub use self::input_coercion::coerce_variable_values;
pub use self::input_coercion::InputCoercionError;
pub use self::introspection_execute::SchemaIntrospectionQuery;
pub use self::introspection_split::SchemaIntrospectionError;
pub use self::introspection_split::SchemaIntrospectionSplit;
pub use self::response::GraphQLError;
pub use self::response::Response;
pub use self::response::ResponseData;
pub use self::response::ResponseDataPathElement;
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
