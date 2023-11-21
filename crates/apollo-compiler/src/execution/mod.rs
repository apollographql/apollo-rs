//! APIs related to [executing a GraphQL request][execution]
//! and returning a [GraphQL response][response]
//!
//! [execution]: https://spec.graphql.org/October2021/#sec-Execution
//! [response]: https://spec.graphql.org/October2021/#sec-Response

/// A source location (line and column numbers) for a [`GraphQLError`].
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct GraphQLLocation {
    /// The line number for this location, starting at 1 for the first line.
    pub line: usize,
    /// The column number for this location, starting at 1 and counting characters (Unicode Scalar
    /// Values) like [`str::chars`].
    pub column: usize,
}

/// A serializable error, as found in a GraphQL response.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct GraphQLError {
    /// The error message.
    pub message: String,

    /// Locations relevant to the error, if any.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub locations: Vec<GraphQLLocation>,
}
