use crate::execution::JsonMap;
use serde::Deserialize;
use serde::Serialize;

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
