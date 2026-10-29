use crate::definition::TypeDefinition;
use crate::DocumentBuilder;
use apollo_compiler::ast;
use apollo_compiler::ast::DirectiveLocation;
use apollo_compiler::Name;
use arbitrary::Result as ArbitraryResult;
use indexmap::IndexSet;

impl DocumentBuilder<'_> {
    /// Generate a scalar definition or extension.
    pub fn scalar_type_definition(&mut self) -> ArbitraryResult<ast::Definition> {
        let mut extend = !self.scalar_type_defs.is_empty() && self.u.arbitrary().unwrap_or(false);
        let mut name = if extend {
            let available_scalars: Vec<&Name> = self
                .scalar_type_defs
                .iter()
                .filter_map(|scalar| {
                    if scalar.is_extension_definition() {
                        None
                    } else {
                        Some(scalar.type_name())
                    }
                })
                .collect();
            (*self.u.choose(&available_scalars)?).clone()
        } else {
            self.type_name()?
        };
        let description = self.optional_description()?;
        let applied: IndexSet<Name> = self
            .scalar_type_defs
            .iter()
            .filter(|def| def.type_name() == &name)
            .flat_map(|def| {
                def.directives()
                    .iter()
                    .map(|directive| directive.name.clone())
            })
            .collect();
        let mut directives = self.type_directives(DirectiveLocation::Scalar, &applied)?;
        self.drop_directives_reaching(&mut directives, &name);

        // A scalar extension must add at least one directive. If we picked the extend path
        // but have no directives to add, fall back to a fresh unique scalar instead of
        // emitting a duplicate definition.
        if extend && directives.is_empty() {
            extend = false;
            name = self.type_name()?;
        }

        Ok(if extend {
            ast::ScalarTypeExtension { name, directives }.into()
        } else {
            ast::ScalarTypeDefinition {
                description,
                name,
                directives,
            }
            .into()
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::test_support::generated;
    use crate::test_support::prop_assert_valid;
    use proptest::prelude::*;

    const SCHEMA: &str = "schema { query: Query }
        directive @d(x: Int, y: [String!]!, t: T) on SCALAR
        type Query { s: S }
        scalar S @d(y: [])
        scalar T @specifiedBy(url: \"https://example.com\")";

    proptest! {
        #[test]
        fn generates_valid_definitions(case in generated(SCHEMA, |b| b.scalar_type_definition().map(Some))) {
            if let Some(doc) = &case.document {
                prop_assert_valid(doc)?;
            }
        }
    }
}
