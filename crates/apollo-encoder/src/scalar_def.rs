use std::fmt;

use crate::StringValue;
/// Represents scalar types such as Int, String, and Boolean.
/// Scalars cannot have fields.
///
/// *ScalarTypeDefinition*:
///     Description? **scalar** Name Directives?
///
/// Detailed documentation can be found in [GraphQL spec](https://spec.graphql.org/October2021/#sec-Scalar).
#[derive(Debug, PartialEq, Clone)]
pub struct ScalarDef {
    // Name must return a String.
    name: String,
    // Description may return a String or null.
    description: StringValue,
}

/// ### Example
/// ```rust
/// use apollo_encoder::ScalarDefBuilder;
///
/// let scalar = ScalarDefBuilder::new("NumberOfTreatsPerDay")
///     .description("Int representing number of treats received.")
///     .build();
///
/// assert_eq!(
///     scalar.to_string(),
///     r#""Int representing number of treats received."
/// scalar NumberOfTreatsPerDay
/// "#
/// );
/// ```
#[derive(Debug, Clone)]
pub struct ScalarDefBuilder {
    // Name must return a String.
    name: String,
    // Description may return a String or null.
    description: Option<String>,
}

impl ScalarDefBuilder {
    /// Create a new instance of ScalarDefBuilder.
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            description: None,
        }
    }

    /// Set the ScalarDef's description.
    pub fn description(mut self, description: &str) -> Self {
        self.description = Some(description.to_string());
        self
    }

    /// Create a new instance of ScalarDef.
    pub fn build(self) -> ScalarDef {
        ScalarDef {
            name: self.name,
            description: StringValue::Top {
                source: self.description,
            },
        }
    }
}

impl fmt::Display for ScalarDef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.description)?;
        writeln!(f, "scalar {}", self.name)
    }
}

#[cfg(test)]
mod tests {
    use crate::ScalarDefBuilder;
    use pretty_assertions::assert_eq;

    #[test]
    fn it_encodes_scalar() {
        let scalar = ScalarDefBuilder::new("NumberOfTreatsPerDay").build();

        assert_eq!(
            scalar.to_string(),
            r#"scalar NumberOfTreatsPerDay
"#
        );
    }

    #[test]
    fn it_encodes_scalar_with_description() {
        let scalar = ScalarDefBuilder::new("NumberOfTreatsPerDay")
            .description("Int representing number of treats received.")
            .build();

        assert_eq!(
            scalar.to_string(),
            r#""Int representing number of treats received."
scalar NumberOfTreatsPerDay
"#
        );
    }
}
