//! GraphQL [responses](https://spec.graphql.org/draft/#sec-Response)
//!
//! This exists primarily to support [`introspection::partial_execute`].

#[cfg(doc)]
use crate::introspection;
use crate::parser::LineColumn;
use crate::parser::SourceMap;
use crate::parser::SourceSpan;
use serde::Deserialize;
use serde::Serialize;
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

/// A [response](https://spec.graphql.org/October2021/#sec-Response-Format)
/// to a GraphQL request that did not cause any [request error][crate::request::RequestError]
/// and started [execution](https://spec.graphql.org/draft/#sec-Execution)
/// of selection sets and fields.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExecutionResponse {
    // <https://spec.graphql.org/October2021/#note-6f005> suggests serializing this first
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub errors: Vec<GraphQLError>,

    pub data: Option<JsonMap>,
}

/// A serializable [error](https://spec.graphql.org/October2021/#sec-Errors.Error-result-format),
/// as found in a GraphQL response.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GraphQLError {
    /// The error message.
    pub message: String,

    /// Locations in relevant to the error, if any.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub locations: Vec<LineColumn>,

    /// If non-empty, the error is a [field error]
    /// for the particular field found at this path in [`ExecutionResponse::data`].
    ///
    /// [field error]: https://spec.graphql.org/October2021/#sec-Errors.Field-errors
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub path: Vec<ResponseDataPathSegment>,

    /// Reserved for any additional information
    #[serde(skip_serializing_if = "JsonMap::is_empty")]
    #[serde(default)]
    pub extensions: JsonMap,
}

/// A `Vec<ResponseDataPathSegment>` like in [`GraphQLError::path`]
/// represents a [path](https://spec.graphql.org/draft/#sec-Errors.Error-Result-Format)
/// into [`ExecutionResponse::data`],
/// starting at the root and indexing into increasingly nested JSON objects or arrays.
///
/// # Example
///
/// In a GraphQL response like this:
///
/// ```json
/// {
///   "data": {
///     "players": [
///       {"name": "Alice"},
///       {"name": "Bob"}
///     ]
///   },
///   "errors": [
///     {
///       "message": "Something went wrong",
///       "path": ["players", 1, "name"]
///     }
///   ]
/// }
/// ```
///
/// The error path would have a Rust representation like
/// `vec![Field("players"), ListIndex(1), Field("name")]`
/// and designate the value `"name": "Bob"`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ResponseDataPathSegment {
    /// The relevant key in an object value
    Field(crate::Name),

    /// The index of the relevant item in a list value
    ListIndex(usize),
}

impl GraphQLError {
    pub fn new(
        message: impl Into<String>,
        location: Option<SourceSpan>,
        sources: &SourceMap,
    ) -> Self {
        Self {
            message: message.into(),
            locations: location
                .into_iter()
                .filter_map(|location| location.line_column(sources))
                .collect(),
            path: Default::default(),
            extensions: Default::default(),
        }
    }
}
