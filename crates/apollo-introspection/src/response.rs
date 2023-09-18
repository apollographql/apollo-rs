use super::JsonMap;
use apollo_compiler::schema::Name;
use apollo_compiler::NodeLocation;
use serde::Serialize;

/// <https://spec.graphql.org/October2021/#sec-Response-Format>
#[derive(Debug, Clone, Serialize)]
pub struct Response {
    /// None/null if a field error was propagated all the way to the root
    pub data: Option<JsonMap>,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub errors: Vec<Error>,
}

/// A response that contains a [request error].
///
/// Does not contain a `data` entry. This is different from `data: null`.
///
/// [request error]: https://spec.graphql.org/October2021/#sec-Errors.Request-errors
#[derive(Debug, Clone, Serialize)]
pub struct RequestErrorResponse {
    pub errors: [Error; 1],
}

/// <https://spec.graphql.org/October2021/#sec-Errors.Error-result-format>
#[derive(Debug, Clone, Serialize)]
pub struct Error {
    pub message: String,

    /// Empty for request errors
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub path: Vec<PathElement>,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub locations: Vec<Location>,

    #[serde(skip_serializing_if = "JsonMap::is_empty")]
    pub extensions: JsonMap,
}

/// Possible key in the `Error::extensions` map
pub const EXTENSION_VALIDATION_SHOULD_HAVE_CAUGHT_THIS: &str =
    "APOLLO_VALIDATION_SHOULD_HAVE_CAUGHT_THIS";

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
pub struct Location {
    line: u32,
    column: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PathElement {
    Field(Name),
    ListItem { index: usize },
}

pub(crate) type LinkedPath<'a> = Option<&'a LinkedPathElement<'a>>;

pub(crate) struct LinkedPathElement<'a> {
    pub(crate) element: PathElement,
    pub(crate) next: LinkedPath<'a>,
}

pub(crate) fn request_error(message: impl Into<String>) -> RequestErrorResponse {
    Error {
        message: message.into(),
        path: Vec::new(),
        locations: Vec::new(),
        extensions: JsonMap::new(),
    }
    .into_request_error()
}

pub(crate) fn field_error(
    message: impl Into<String>,
    path: LinkedPath<'_>,
    location: Option<NodeLocation>,
) -> Error {
    Error {
        message: message.into(),
        path: path_to_vec(path),
        locations: to_locations(location),
        extensions: JsonMap::new(),
    }
}

pub(crate) fn path_to_vec(mut link: LinkedPath<'_>) -> Vec<PathElement> {
    let mut path = Vec::new();
    while let Some(node) = link {
        path.push(node.element.clone());
        link = node.next;
    }
    path.reverse();
    path
}

pub(crate) fn to_locations(location: Option<NodeLocation>) -> Vec<Location> {
    location
        .into_iter()
        .filter_map(|_loc| {
            // TODO
            None
        })
        .collect()
}

impl Error {
    pub(crate) fn validation_should_have_caught_this(mut self) -> Self {
        self.extensions
            .insert(EXTENSION_VALIDATION_SHOULD_HAVE_CAUGHT_THIS, true.into());
        self
    }

    pub(crate) fn into_request_error(self) -> RequestErrorResponse {
        RequestErrorResponse { errors: [self] }
    }
}

impl RequestErrorResponse {
    pub(crate) fn validation_should_have_caught_this(mut self) -> Self {
        self.errors[0]
            .extensions
            .insert(EXTENSION_VALIDATION_SHOULD_HAVE_CAUGHT_THIS, true.into());
        self
    }

    pub(crate) fn into_field_error(
        self,
        path: LinkedPath<'_>,
        location: Option<NodeLocation>,
    ) -> Error {
        let [mut err] = self.errors;
        err.path = path_to_vec(path);
        err.locations = to_locations(location);
        err
    }
}

impl Response {
    pub fn merge(mut self, mut other: Self) -> Self {
        if let (Some(self_data), Some(other_data)) = (&mut self.data, other.data) {
            self_data.extend(other_data)
        } else {
            // null was propagated to the root:
            self.data = None
        }
        self.errors.append(&mut other.errors);
        self
    }
}

impl Serialize for PathElement {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            PathElement::Field(name) => name.as_str().serialize(serializer),
            PathElement::ListItem { index } => index.serialize(serializer),
        }
    }
}
