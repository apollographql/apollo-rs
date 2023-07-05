use std::fmt;

use crate::{Directive, StringValue};

/// A GraphQL service’s collective type system capabilities are referred to as
/// that service’s “schema”.
///
/// *SchemaDefinition*:
///     Description? **schema** Directives? **{** RootOperationTypeDefinition* **}**
///
/// Detailed documentation can be found in [GraphQL spec](https://spec.graphql.org/October2021/#sec-Schema).
///
/// ### Example
/// ```rust
/// use apollo_encoder::{SchemaDefinition};
/// use indoc::indoc;
///
/// let mut schema_def = SchemaDefinition::new();
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
pub struct SchemaDefinition {
    // Description may be a String.
    description: Option<StringValue>,
    // The vector of fields in a schema to represent root operation type
    // definition.
    query: Option<String>,
    mutation: Option<String>,
    subscription: Option<String>,
    directives: Vec<Directive>,
    // Extend a schema
    extend: bool,
}

impl SchemaDefinition {
    /// Create a new instance of SchemaDef.
    pub fn new() -> Self {
        Self {
            description: None,
            query: None,
            mutation: None,
            subscription: None,
            directives: Vec::new(),
            extend: false,
        }
    }

    /// Set the SchemaDef's description.
    pub fn description(&mut self, description: String) {
        self.description = Some(StringValue::Top {
            source: description,
        });
    }

    /// Add a directive.
    pub fn directive(&mut self, directive: Directive) {
        self.directives.push(directive);
    }

    /// Set as an extension
    pub fn extend(&mut self) {
        self.extend = true;
    }

    /// Set the schema def's query type.
    pub fn query(&mut self, query: String) {
        self.query = Some(query);
    }

    /// Set the schema def's mutation type.
    pub fn mutation(&mut self, mutation: String) {
        self.mutation = Some(mutation);
    }

    /// Set the schema def's subscription type.
    pub fn subscription(&mut self, subscription: String) {
        self.subscription = Some(subscription);
    }
}

impl Default for SchemaDefinition {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for SchemaDefinition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.extend {
            write!(f, "extend ")?;
        // No description when it's an extension schema
        } else if let Some(description) = &self.description {
            writeln!(f, "{description}")?;
        }

        write!(f, "schema")?;

        for directive in &self.directives {
            write!(f, " {directive}")?;
        }

        if self.query.is_some() || self.mutation.is_some() || self.subscription.is_some() {
            writeln!(f, " {{")?;

            if let Some(query) = &self.query {
                writeln!(f, "  query: {query}")?;
            }

            if let Some(mutation) = &self.mutation {
                writeln!(f, "  mutation: {mutation}")?;
            }

            if let Some(subscription) = &self.subscription {
                writeln!(f, "  subscription: {subscription}")?;
            }

            writeln!(f, "}}")
        } else {
            writeln!(f)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;
    use pretty_assertions::assert_eq;

    #[test]
    fn it_encodes_schema_with_mutation_and_subscription() {
        let mut schema_def = SchemaDefinition::new();
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

    #[test]
    fn it_encodes_extend_schema_with_mutation_and_subscription() {
        let mut schema_def = SchemaDefinition::new();
        schema_def.query("TryingToFindCatQuery".to_string());
        schema_def.mutation("MyMutation".to_string());
        schema_def.subscription("MySubscription".to_string());
        schema_def.extend();

        assert_eq!(
            schema_def.to_string(),
            indoc! { r#"
            extend schema {
              query: TryingToFindCatQuery
              mutation: MyMutation
              subscription: MySubscription
            }
        "#}
        );
    }

    #[test]
    fn it_encodes_extend_schema_with_directives_only() {
        let mut schema_def = SchemaDefinition::new();
        schema_def.directive(Directive::new("foo".to_string()));
        schema_def.extend();

        assert_eq!(
            schema_def.to_string(),
            indoc! { r#"
            extend schema @foo
        "#}
        )
    }
}
