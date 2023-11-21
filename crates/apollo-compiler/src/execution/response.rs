use crate::execution::JsonMap;
use crate::node::NodeLocation;
use crate::SourceMap;
use serde::Deserialize;
use serde::Serialize;

/// This key is set to (JSON) `true` in [`GraphQLError::extensions`]
/// when reaching a situtation that should not happen with a valid schema and document.
///
/// Since the relevant APIs take `Valid<_>` parameters,
/// either apollo-compiler has a validation bug or `Valid::assume_valid` was used incorrectly.
pub const EXTENSION_SUSPECTED_VALIDATION_BUG: &str = "APOLLO_SUSPECTED_VALIDATION_BUG";

/// A [GraphQL response](https://spec.graphql.org/October2021/#sec-Response-Format)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Response {
    // <https://spec.graphql.org/October2021/#note-6f005> suggests serializing this first
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub errors: Vec<GraphQLError>,

    #[serde(skip_serializing_if = "ResponseData::is_absent")]
    #[serde(default = "ResponseData::absent")]
    pub data: ResponseData,

    #[serde(skip_serializing_if = "JsonMap::is_empty")]
    #[serde(default)]
    pub extensions: JsonMap,
}

/// The `data` entry of a [`Response`]
#[derive(Debug, Clone, Deserialize)]
#[serde(from = "Option<JsonMap>")]
pub enum ResponseData {
    /// Execution returned an object.
    /// [`Response::data`] is serialized as a JSON object.
    Object(JsonMap),

    /// Execution encountered a [field error] on a non-null field,
    /// and null was [propagated] all the way to the root of the response.
    /// [`Response::data`] is serialized as JSON null.
    ///
    /// [field error]: https://spec.graphql.org/October2021/#sec-Errors.Field-errors
    /// [propagated]: https://spec.graphql.org/October2021/#sec-Handling-Field-Errors
    Null,

    /// A [request error] was encountered. Execution did not start.
    /// [`Response::data`] is skipped from serialization.
    ///
    /// [request error]: https://spec.graphql.org/October2021/#sec-Errors.Request-errors
    Absent,
}

/// A [request error] that aborted the handling of a request before execution started.
///
/// [`RequestError`] and [`Result`]`<`[`Response`]`, `[`RequestError`]`>`
/// can be [converted][std::convert] to [`Response`].
///
/// [request error]: https://spec.graphql.org/October2021/#sec-Errors.Request-errors
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct RequestError(pub GraphQLError);

/// A serializable [error](https://spec.graphql.org/October2021/#sec-Errors.Error-result-format),
/// as found in a GraphQL [response][Response].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GraphQLError {
    /// The error message.
    pub message: String,

    /// Locations in relevant to the error, if any.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub locations: Vec<GraphQLLocation>,

    /// If non-empty, the error is a [field error]
    /// for the particular field found at this path in [`Response::data`].
    ///
    /// [field error]: https://spec.graphql.org/October2021/#sec-Errors.Field-errors
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub path: Vec<PathElement>,

    #[serde(skip_serializing_if = "JsonMap::is_empty")]
    #[serde(default)]
    pub extensions: JsonMap,
}

/// A source location (line and column numbers) for a [`GraphQLError`].
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GraphQLLocation {
    /// The line number for this location, starting at 1 for the first line.
    pub line: usize,
    /// The column number for this location, starting at 1 and counting characters (Unicode Scalar
    /// Values) like [`str::chars`].
    pub column: usize,
}

/// An element of [`GraphQLError::path`]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PathElement {
    /// The relevant key in an object value
    Field(crate::ast::Name),

    /// The index of the relevant item in a list value
    ListIndex(usize),
}

impl GraphQLError {
    /// Call for errors that should not happen with a valid schema and document.
    /// See [`EXTENSION_SUSPECTED_VALIDATION_BUG`].
    pub fn validation_bug(mut self) -> Self {
        self.extensions
            .insert(EXTENSION_SUSPECTED_VALIDATION_BUG, true.into());
        self
    }
}

impl RequestError {
    /// Call for errors that should not happen with a valid schema and document.
    /// See [`EXTENSION_SUSPECTED_VALIDATION_BUG`].
    pub fn validation_bug(self) -> Self {
        Self(self.0.validation_bug())
    }
}

impl GraphQLLocation {
    /// Convert a `NodeLocation` to a line and column number
    pub fn from_node(sources: &SourceMap, location: Option<NodeLocation>) -> Option<Self> {
        let loc = location?;
        let source = sources.get(&loc.file_id)?;
        source
            .get_line_column(loc.offset())
            .map(|(line, column)| GraphQLLocation {
                line: line + 1,
                column: column + 1,
            })
    }
}

impl ResponseData {
    /// For serde `skip_serializing_if`
    fn is_absent(&self) -> bool {
        matches!(self, Self::Absent)
    }

    /// For serde `default`
    fn absent() -> Self {
        Self::Absent
    }
}

impl Serialize for ResponseData {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            ResponseData::Absent => panic!("ResponseData::None should not be serialized"),
            ResponseData::Null => serializer.serialize_unit(),
            ResponseData::Object(map) => map.serialize(serializer),
        }
    }
}

impl From<Option<JsonMap>> for ResponseData {
    fn from(value: Option<JsonMap>) -> Self {
        if let Some(data) = value {
            Self::Object(data)
        } else {
            Self::Null
        }
    }
}

impl From<RequestError> for Response {
    fn from(error: RequestError) -> Self {
        Self {
            errors: vec![error.0],
            data: ResponseData::Absent,
            extensions: JsonMap::new(),
        }
    }
}

impl From<Result<Response, RequestError>> for Response {
    fn from(result: Result<Response, RequestError>) -> Self {
        result.unwrap_or_else(|request_error| request_error.into())
    }
}

impl RequestError {
    pub fn new(message: impl ToString) -> Self {
        Self(GraphQLError {
            message: message.to_string(),
            locations: Default::default(),
            path: Default::default(),
            extensions: Default::default(),
        })
    }
}
