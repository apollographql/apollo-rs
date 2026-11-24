use crate::DocumentBuilder;
use apollo_compiler::ast;
use apollo_compiler::ast::DirectiveLocation;
use apollo_compiler::ast::Type;
use apollo_compiler::Node;
use arbitrary::Result as ArbitraryResult;

impl DocumentBuilder<'_> {
    /// Generate a compiler schema definition.
    pub fn schema_definition(&mut self) -> ArbitraryResult<ast::SchemaDefinition> {
        let description = self.optional_description()?;
        let directives = self.directives(DirectiveLocation::Schema)?;
        let mut named_types: Vec<Type> = self
            .list_existing_object_types()
            .into_iter()
            .filter(|ty| ty.is_named() && !crate::ty::is_builtin(ty))
            .collect();

        // Every schema must declare a `query` root operation type, even
        // when no operations reference it. Mutation and subscription are
        // optional.
        //
        // See <https://spec.graphql.org/September2025/#sec-Root-Operation-Types>.
        let query = Some(self.u.choose(&named_types)?.clone());

        // Ensure we choose a different type for other roots
        named_types.retain(|ty| Some(ty) != query.as_ref());

        let include_mutation = !named_types.is_empty() && self.u.arbitrary().unwrap_or(false);
        let mutation = if include_mutation {
            let m = self.u.choose(&named_types)?.clone();
            named_types.retain(|ty| *ty != m);
            Some(m)
        } else {
            None
        };

        let include_subscription = !named_types.is_empty() && self.u.arbitrary().unwrap_or(false);
        let subscription = if include_subscription {
            Some(self.u.choose(&named_types)?.clone())
        } else {
            None
        };

        Ok(ast::SchemaDefinition {
            description,
            directives,
            root_operations: [
                (ast::OperationType::Query, query),
                (ast::OperationType::Mutation, mutation),
                (ast::OperationType::Subscription, subscription),
            ]
            .into_iter()
            .filter_map(|(operation, ty)| {
                ty.map(|ty| Node::new((operation, ty.inner_named_type().clone())))
            })
            .collect(),
        })
    }
}
