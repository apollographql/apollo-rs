use std::fmt;

use crate::{Directive, StringValue};
/// Represents scalar types such as Int, String, and Boolean.
/// Scalars cannot have fields.
///
/// *ScalarTypeDefinition*:
///     Description? **scalar** Name Directives?
///
/// Detailed documentation can be found in [GraphQL spec](https://spec.graphql.org/October2021/#sec-Scalar).
/// ### Example
/// ```rust
/// use apollo_encoder::ScalarDefinition;
///
/// let mut scalar = ScalarDefinition::new("NumberOfTreatsPerDay".to_string());
/// scalar.description(
///     "Int representing number of treats received.".to_string(),
/// );
///
/// assert_eq!(
///     scalar.to_string(),
///     r#""Int representing number of treats received."
/// scalar NumberOfTreatsPerDay
/// "#
/// );
/// ```
#[derive(Debug, PartialEq, Clone)]
pub struct ScalarDefinition {
    // Name must return a String.
    name: String,
    // Description may return a String or null.
    description: Option<StringValue>,
    directives: Vec<Directive>,
    extend: bool,
}

impl ScalarDefinition {
    /// Create a new instance of Scalar Definition.
    pub fn new(name: String) -> Self {
        Self {
            name,
            description: None,
            directives: Vec::new(),
            extend: false,
        }
    }

    /// Set the ScalarDef's description.
    pub fn description(&mut self, description: String) {
        self.description = Some(StringValue::Top {
            source: description,
        });
    }

    /// Add a directive.
    pub fn directive(&mut self, directive: Directive) {
        self.directives.push(directive);
    }

    /// Set the scalar as an extension
    pub fn extend(&mut self) {
        self.extend = true;
    }
}

impl fmt::Display for ScalarDefinition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.extend {
            write!(f, "extend ")?;
        } else if let Some(description) = &self.description {
            writeln!(f, "{description}")?;
        }

        write!(f, "scalar {}", self.name)?;
        for directive in &self.directives {
            write!(f, " {directive}")?;
        }
        writeln!(f)
    }
}

#[cfg(test)]
mod tests {

    use crate::{Argument, Value};

    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn it_encodes_scalar() {
        let scalar = ScalarDefinition::new("NumberOfTreatsPerDay".to_string());
        assert_eq!(
            scalar.to_string(),
            r#"scalar NumberOfTreatsPerDay
"#
        );
    }

    #[test]
    fn it_encodes_scalar_with_description() {
        let mut scalar = ScalarDefinition::new("NumberOfTreatsPerDay".to_string());
        scalar.description("Int representing number of treats received.".to_string());

        assert_eq!(
            scalar.to_string(),
            r#""Int representing number of treats received."
scalar NumberOfTreatsPerDay
"#
        );
    }

    #[test]
    fn it_encodes_scalar_with_extend_directive() {
        let mut scalar = ScalarDefinition::new("NumberOfTreatsPerDay".to_string());
        scalar.description("Int representing number of treats received.".to_string());
        scalar.extend();
        let mut directive = Directive::new(String::from("tag"));
        directive.arg(Argument::new(
            String::from("name"),
            Value::String("team-admin".to_string()),
        ));
        scalar.directive(directive);

        assert_eq!(
            scalar.to_string(),
            r#"extend scalar NumberOfTreatsPerDay @tag(name: "team-admin")
"#
        );
    }
}
