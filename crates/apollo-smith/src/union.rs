use crate::definition::TypeDefinition;
use crate::DocumentBuilder;
use apollo_compiler::ast;
use apollo_compiler::ast::DirectiveLocation;
use apollo_compiler::Name;
use arbitrary::Result as ArbitraryResult;
use indexmap::IndexSet;

impl DocumentBuilder<'_> {
    /// Generate an union definition or extension.
    pub fn union_type_definition(&mut self) -> ArbitraryResult<ast::Definition> {
        let extend = !self.union_type_defs.is_empty() && self.u.arbitrary().unwrap_or(false);
        let name = if extend {
            let available_unions: Vec<&Name> = self
                .union_type_defs
                .iter()
                .filter_map(|union| {
                    if union.is_extension_definition() {
                        None
                    } else {
                        Some(union.type_name())
                    }
                })
                .collect();
            (*self.u.choose(&available_unions)?).clone()
        } else {
            self.type_name()?
        };
        let description = self.optional_description()?;
        let applied: IndexSet<Name> = self
            .union_type_defs
            .iter()
            .filter(|def| def.type_name() == &name)
            .flat_map(|def| {
                def.directives()
                    .iter()
                    .map(|directive| directive.name.clone())
            })
            .collect();
        let directives = self.type_directives(DirectiveLocation::Union, &applied)?;
        // Union members must be Object base types — built-in scalars,
        // other unions, interfaces, enums, and input objects are all
        // invalid as members.
        //
        // See <https://spec.graphql.org/September2025/#sec-Unions>.
        let existing_members = self.union_members(&name);
        let object_types: Vec<_> = self
            .list_existing_object_types()
            .into_iter()
            .filter(|o| !existing_members.contains(o.inner_named_type()))
            .collect();
        if object_types.is_empty() {
            return Err(arbitrary::Error::IncorrectFormat);
        }
        let members = (0..self.u.int_in_range(2..=10)?)
            .map(|_| Ok(self.u.choose(&object_types)?.inner_named_type().clone()))
            .collect::<ArbitraryResult<IndexSet<_>>>()?;

        if extend && directives.is_empty() && members.is_empty() {
            return Err(arbitrary::Error::IncorrectFormat);
        }

        Ok(if extend {
            ast::UnionTypeExtension {
                name,
                directives,
                members: members.into_iter().collect(),
            }
            .into()
        } else {
            ast::UnionTypeDefinition {
                description,
                name,
                directives,
                members: members.into_iter().collect(),
            }
            .into()
        })
    }

    /// Members of a union across its base definition and extensions, so
    /// selections can narrow to any member, not only those listed on the
    /// base definition.
    pub(crate) fn union_members(&self, name: &Name) -> IndexSet<Name> {
        self.union_type_defs
            .iter()
            .filter(|u| u.type_name() == name)
            .flat_map(|u| u.members().iter().cloned())
            .collect()
    }
}

// Unions declare no fields, so selections on them are limited to
// `__typename` and fragments on their members.

#[cfg(test)]
mod tests {
    use crate::test_support::generated;
    use crate::test_support::prop_assert_valid;
    use proptest::prelude::*;

    const SCHEMA: &str = "schema { query: Query }
        directive @d(x: Int, y: [String!]!) on UNION
        type Query { u: U }
        union U @d(y: []) = A | B
        type A { a: Int }
        type B { b: Int }
        type C { c: Int }";

    proptest! {
        #[test]
        fn generates_valid_definitions(case in generated(SCHEMA, |b| b.union_type_definition().map(Some))) {
            if let Some(doc) = &case.document {
                prop_assert_valid(doc)?;
            }
        }
    }
}
