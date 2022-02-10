use arbitrary::Result;

use crate::DocumentBuilder;

/// The `__Description` type represents a description
///
/// *Description*:
///     "string"
///
/// Detailed documentation can be found in [GraphQL spec](https://spec.graphql.org/October2021/#sec-Descriptions).
///
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Description(String);

impl From<Description> for String {
    fn from(desc: Description) -> Self {
        desc.0
    }
}

impl Description {
    pub(crate) fn new(desc: String) -> Self {
        Description(desc)
    }
}

impl<'a> DocumentBuilder<'a> {
    /// Create an arbitrary `Description`
    pub fn description(&mut self) -> Result<Description> {
        Ok(Description::new(self.limited_string(50)?))
    }
}
