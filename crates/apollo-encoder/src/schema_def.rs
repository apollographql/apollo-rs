use std::fmt;

use crate::StringValue;

/// A GraphQL service’s collective type system capabilities are referred to as that service’s “schema”.
///
/// *SchemaDefinition*:
///     Description? **schema** Directives? **{** RootOperationTypeDefinition* **}**
///
/// Detailed documentation can be found in [GraphQL spec](https://spec.graphql.org/October2021/#sec-Schema).
///
/// ### Example
/// ```rust
/// use apollo_encoder::{SchemaDef};
/// use indoc::indoc;
///
/// let mut schema_def = SchemaDef::new();
/// schema_def.query("TryingToFindCatQuery".to_string());
/// schema_def.mutation("MyMutation".to_string());
/// schema_def.subscription("MySubscription".to_string());
///
/// assert_eq!(
///    schema_def.to_string(),
///    indoc! { r#"
///        schema {
///          query: TryingToFindCatQuery
///          mutation: MyMutation
///          subscription: MySubscription
///        }
///    "#}
/// );
/// ```

#[derive(Debug, Clone)]
pub struct SchemaDef {
    // Description may be a String.
    description: StringValue,
    // The vector of fields in a schema to represent root operation type
    // definition.
    query: Option<String>,
    mutation: Option<String>,
    subscription: Option<String>,
}

impl SchemaDef {
    /// Create a new instance of SchemaDef.
    pub fn new() -> Self {
        Self {
            description: StringValue::Top { source: None },
            query: None,
            mutation: None,
            subscription: None,
        }
    }

    /// Set the SchemaDef's description.
    pub fn description(&mut self, description: &str) {
        self.description = StringValue::Top {
            source: Some(description.to_owned()),
        };
    }

    /// Set the schema def's query type.
    pub fn query(&mut self, query: &str) {
        self.query = Some(query.to_owned());
    }

    /// Set the schema def's mutation type.
    pub fn mutation(&mut self, mutation: &str) {
        self.mutation = Some(mutation.to_owned());
    }

    /// Set the schema def's subscription type.
    pub fn subscription(&mut self, subscription: &str) {
        self.subscription = Some(subscription.to_owned());
    }
}

impl Default for SchemaDef {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for SchemaDef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.description)?;

        writeln!(f, "schema {{")?;
        if let Some(query) = &self.query {
            writeln!(f, "  query: {}", query)?;
        }

        if let Some(mutation) = &self.mutation {
            writeln!(f, "  mutation: {}", mutation)?;
        }

        if let Some(subscription) = &self.subscription {
            writeln!(f, "  subscription: {}", subscription)?;
        }

        writeln!(f, "}}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;
    use pretty_assertions::assert_eq;

    #[test]
    fn it_encodes_schema_with_mutation_and_subscription() {
        let mut schema_def = SchemaDef::new();
        schema_def.query("TryingToFindCatQuery".to_string());
        schema_def.mutation("MyMutation".to_string());
        schema_def.subscription("MySubscription".to_string());

        assert_eq!(
            schema_def.to_string(),
            indoc! { r#"
            schema {
              query: TryingToFindCatQuery
              mutation: MyMutation
              subscription: MySubscription
            }
        "#}
        );
    }
}
