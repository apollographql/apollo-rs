use crate::execution::engine::path_to_vec;
use crate::execution::engine::LinkedPath;
use crate::execution::engine::PropagateNull;
use crate::execution::JsonMap;
use crate::node::NodeLocation;
use crate::SourceMap;
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

    /// Reserved for any additional information
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

    /// Reserved for any additional information
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

/// Returned as an error for situtations that should not happen with a valid schema or document.
///
/// Since the relevant APIs take [`Valid<_>`][crate::validation::Valid] parameters,
/// either apollo-compiler has a validation bug
/// or [`assume_valid`][crate::validation::Valid::assume_valid] was used incorrectly.
///
/// Can be [converted][std::convert] to [`GraphQLError`],
/// which populates [`extensions`][GraphQLError::extensions]
/// with a `"APOLLO_SUSPECTED_VALIDATION_BUG": true` entry.
#[derive(Debug, Clone)]
pub struct SuspectedValidationBug {
    pub message: String,
    pub location: Option<NodeLocation>,
}

impl Response {
    /// Create a response for a [request error]:
    /// handling of a request was aborted before execution started.
    ///
    /// [request error]: https://spec.graphql.org/October2021/#sec-Errors.Request-errors
    pub fn from_request_error(error: impl Into<GraphQLError>) -> Self {
        Self {
            errors: vec![error.into()],
            data: ResponseData::Absent,
            extensions: JsonMap::new(),
        }
    }

    /// Merge two responses into one, such as to handle
    /// [`SchemaIntrospection::Both`][crate::execution::SchemaIntrospection::Both].
    pub fn merge(mut self, mut other: Self) -> Self {
        match (&mut self.data, other.data) {
            (ResponseData::Absent, _) | (_, ResponseData::Absent) => {
                // If either side is a request error (absent data), return a request error
                self.data = ResponseData::Absent
            }
            (ResponseData::Null, _) | (_, ResponseData::Null) => {
                // Otherwise if either side propagated null from a field error
                // to the root of the response, return null data.
                self.data = ResponseData::Null
            }
            (ResponseData::Object(self_data), ResponseData::Object(other_data)) => {
                // Merge two objects/maps
                self_data.extend(other_data)
            }
        }
        self.errors.append(&mut other.errors);
        self.extensions.extend(other.extensions);
        self
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
        match value {
            Some(data) => Self::Object(data),
            None => Self::Null,
        }
    }
}

impl From<Result<JsonMap, PropagateNull>> for ResponseData {
    fn from(result: Result<JsonMap, PropagateNull>) -> Self {
        match result {
            Ok(data) => Self::Object(data),
            Err(PropagateNull) => Self::Null,
        }
    }
}

impl SuspectedValidationBug {
    pub fn into_graphql_error(self, sources: &SourceMap) -> GraphQLError {
        let Self { message, location } = self;
        let mut err = GraphQLError {
            message,
            locations: GraphQLLocation::from_node(sources, location)
                .into_iter()
                .collect(),
            path: Vec::new(),
            extensions: Default::default(),
        };
        err.extensions
            .insert("APOLLO_SUSPECTED_VALIDATION_BUG", true.into());
        err
    }

    pub(crate) fn into_field_error(
        self,
        sources: &SourceMap,
        path: LinkedPath<'_>,
    ) -> GraphQLError {
        let mut err = self.into_graphql_error(sources);
        err.path = path_to_vec(path);
        err
    }
}
