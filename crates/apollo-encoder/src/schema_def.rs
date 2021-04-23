use std::fmt::{self, Display};

/// A GraphQL service’s collective type system capabilities are referred to as that service’s “schema”.
///
/// *SchemaDefinition*:
///     Description<sub>opt</sub> **schema** Directives<sub>\[Const\] opt</sub> **{** RootOperationTypeDefinition<sub>list</sub> **}**
///
/// Detailed documentation can be found in [GraphQL spec](https://spec.graphql.org/draft/#sec-Schema).
///
/// ### Example
/// ```rust
/// use sdl_encoder::{SchemaDef};
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
    description: Option<String>,
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
            description: None,
            query: None,
            mutation: None,
            subscription: None,
        }
    }

    /// Set the SchemaDef's description.
    pub fn description(&mut self, description: Option<String>) {
        self.description = description
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

impl Default for SchemaDef {
    fn default() -> Self {
        Self::new()
    }
}

impl Display for SchemaDef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(description) = &self.description {
            // We are determing on whether to have description formatted as
            // a multiline comment based on whether or not it already includes a
            // \n.
            match description.contains('\n') {
                true => writeln!(f, "\"\"\"\n{}\n\"\"\"", description)?,
                false => writeln!(f, "\"\"\"{}\"\"\"", description)?,
            }
        }
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
