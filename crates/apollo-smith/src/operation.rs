use crate::variable::OperationVariables;
use crate::DocumentBuilder;
use apollo_compiler::ast::DirectiveLocation;
use apollo_compiler::ast::OperationDefinition;
use apollo_compiler::ast::OperationType;
use apollo_compiler::ast::Selection;
use apollo_compiler::Node;
use arbitrary::Result as ArbitraryResult;

impl DocumentBuilder<'_> {
    /// Generate an operation using the document's schema roots.
    pub fn operation_definition(&mut self) -> ArbitraryResult<Option<OperationDefinition>> {
        self.operation_definition_in_document(false)
    }

    /// Like [`operation_definition`], but forces the operation to be
    /// named when `require_named` is true. A document with more than
    /// one operation may not contain an anonymous operation.
    ///
    /// Operation Name Uniqueness across the document is upheld by
    /// `type_name`, which never returns a name that already exists on
    /// the builder.
    ///
    /// See <https://spec.graphql.org/September2025/#sec-Lone-Anonymous-Operation>
    /// and <https://spec.graphql.org/September2025/#sec-Operation-Name-Uniqueness>.
    ///
    /// [`operation_definition`]: Self::operation_definition
    pub(crate) fn operation_definition_in_document(
        &mut self,
        require_named: bool,
    ) -> ArbitraryResult<Option<OperationDefinition>> {
        let available_operations = self.operation_roots();
        if available_operations.is_empty() {
            return Ok(None);
        }

        let want_name = require_named || self.u.arbitrary().unwrap_or(false);
        let name = if want_name {
            Some(self.type_name()?)
        } else {
            None
        };

        let (operation_type, chosen_ty) = self.u.choose(&available_operations)?;
        let directive_location = match operation_type {
            OperationType::Query => DirectiveLocation::Query,
            OperationType::Mutation => DirectiveLocation::Mutation,
            OperationType::Subscription => DirectiveLocation::Subscription,
        };
        let directives = self.directives(directive_location)?;

        // Stack
        self.stack_ty(&apollo_compiler::ast::Type::Named(chosen_ty.clone()));
        self.operation_variables = Some(OperationVariables::default());

        let (selection_set, _) = self.tracking_merges(|b| {
            if matches!(operation_type, OperationType::Subscription) {
                // Subscription operations must have exactly one root field
                // per <https://spec.graphql.org/September2025/#sec-Single-Root-Field>.
                // We generate exactly one field to satisfy that condition. The 0-index
                // is sometimes used to create an alias for the field.
                Ok(vec![Selection::Field(Node::new(
                    b.field_with_conditions(0, false)?,
                ))])
            } else {
                b.selection_set()
            }
        });

        self.stack.pop();
        let variables = self.operation_variables.take().unwrap_or_default();
        let selection_set = selection_set?;
        assert!(
            self.stack.is_empty(),
            "the stack must be empty at the end of an operation definition"
        );

        let variable_definitions = variables.definitions;

        Ok(Some(OperationDefinition {
            description: None,
            operation_type: *operation_type,
            name,
            variables: variable_definitions.into_iter().map(Node::new).collect(),
            directives,
            selection_set,
        }))
    }
}

#[cfg(test)]
mod tests {
    use crate::test_support::generate_definition;
    use crate::test_support::generated;
    use crate::test_support::input;
    use crate::test_support::prop_assert_valid;
    use crate::test_support::Generated;
    use crate::ArbitraryDocument;
    use crate::DocumentBuilder;
    use apollo_compiler::ast;
    use arbitrary::Arbitrary;
    use arbitrary::Unstructured;
    use proptest::prelude::*;
    use std::collections::HashSet;

    #[test]
    fn generates_operations_for_extended_schema_roots() {
        let schema = ast::Document::parse(
            "schema { query: Q }
             extend schema { mutation: M, subscription: S }
             type Q { queryField: Int }
             type M { mutationField: Int }
             type S { subscriptionField: Int }",
            "schema.graphql",
        )
        .expect("parse extended schema");
        let mut generated = HashSet::new();
        for seed in 0..64 {
            let data: Vec<u8> = (0..512).map(|index| (index + seed) as u8).collect();
            let mut u = Unstructured::new(&data);
            let mut builder =
                DocumentBuilder::with_document(&mut u, schema.clone()).expect("seed builder");
            let operation = builder
                .operation_definition()
                .expect("generate operation")
                .expect("schema has roots");
            generated.insert(operation.operation_type);
            let mut document = schema.clone();
            document.definitions.push(operation.into());
            document
                .to_mixed_validate()
                .expect("valid operation for extended schema");
        }
        assert_eq!(generated.len(), 3);
    }

    const SCHEMA: &str = "schema { query: Query, mutation: Mutation, subscription: Subscription }
        directive @d(x: Int, y: [String!]!) on QUERY | MUTATION | SUBSCRIPTION | FIELD | FRAGMENT_SPREAD | INLINE_FRAGMENT | VARIABLE_DEFINITION
        type Query { a: Int, q: Query }
        type Mutation { m(x: Int): Query }
        type Subscription { s: Query, t: Int }";

    // An operation generated from `data` against a schema generated from
    // `schema_data`, without the schema's own operations. Shown with the
    // schema, since it differs between cases.
    fn operation_against_generated_schema(schema_data: &[u8], data: &[u8]) -> Generated {
        let schema = ArbitraryDocument::arbitrary_take_rest(Unstructured::new(schema_data))
            .ok()
            .map(|doc| {
                let mut ast = doc.0;
                ast.definitions
                    .retain(|d| d.as_operation_definition().is_none());
                ast.to_string()
            });
        let document = schema.and_then(|schema| {
            let operation = generate_definition(&schema, data, |b| b.operation_definition())?;
            Some(format!("{schema}\n{operation}"))
        });
        Generated::new(document.clone(), document)
    }

    proptest! {
        #[test]
        fn generates_valid_operations(case in generated(SCHEMA, |b| b.operation_definition())) {
            if let Some(doc) = &case.document {
                prop_assert_valid(doc)?;
            }
        }

        #[test]
        fn generates_valid_operations_against_generated_schemas(
            case in (input(), input()).prop_map(|(schema_data, data)| {
                operation_against_generated_schema(&schema_data, &data)
            }),
        ) {
            if let Some(doc) = &case.document {
                prop_assert_valid(doc)?;
            }
        }
    }
}
