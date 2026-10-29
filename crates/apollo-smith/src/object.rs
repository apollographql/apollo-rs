use crate::definition::TypeDefinition;
use crate::interface::field_signatures_for;
use crate::DocumentBuilder;
use apollo_compiler::ast;
use apollo_compiler::ast::DirectiveLocation;
use apollo_compiler::Name;
use arbitrary::Result as ArbitraryResult;
use indexmap::IndexSet;

impl DocumentBuilder<'_> {
    /// Generate an object definition or extension.
    pub fn object_type_definition(&mut self) -> ArbitraryResult<ast::Definition> {
        let extend = !self.object_type_defs.is_empty() && self.u.arbitrary().unwrap_or(false);
        let description = self.optional_description()?;
        let name = if extend {
            let available_objects: Vec<&Name> = self
                .object_type_defs
                .iter()
                .filter_map(|object| {
                    if object.is_extension_definition() {
                        None
                    } else {
                        Some(object.type_name())
                    }
                })
                .collect();
            (*self.u.choose(&available_objects)?).clone()
        } else {
            self.type_name()?
        };

        // Extensions can add new `implements` clauses but must not
        // duplicate prior picks or overwrite a field signature the
        // type already commits to.
        let existing_field_signatures = field_signatures_for(&self.object_type_defs, &name);
        let implements_interfaces =
            self.additional_implements(&existing_field_signatures, Some(&name))?;
        let fields_def =
            self.fields_implementing(&name, &implements_interfaces, &existing_field_signatures)?;
        let applied: IndexSet<Name> = self
            .object_type_defs
            .iter()
            .filter(|def| def.type_name() == &name)
            .flat_map(|def| {
                def.directives()
                    .iter()
                    .map(|directive| directive.name.clone())
            })
            .collect();
        let directives = self.type_directives(DirectiveLocation::Object, &applied)?;

        if extend
            && directives.is_empty()
            && fields_def.is_empty()
            && implements_interfaces.is_empty()
        {
            return Err(arbitrary::Error::IncorrectFormat);
        }

        Ok(if extend {
            ast::ObjectTypeExtension {
                name,
                directives,
                implements_interfaces: implements_interfaces.into_iter().collect(),
                fields: fields_def,
            }
            .into()
        } else {
            ast::ObjectTypeDefinition {
                description,
                name,
                directives,
                implements_interfaces: implements_interfaces.into_iter().collect(),
                fields: fields_def,
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
        directive @d(x: Int, y: [String!]!) on OBJECT | FIELD_DEFINITION | ARGUMENT_DEFINITION
        interface I { x: Int }
        interface J { y(z: Int): String }
        type Query { a: A }
        type A implements I @d(y: []) { x: Int, a: A }
        type B { y(z: Int): String }";

    proptest! {
        #[test]
        fn generates_valid_definitions(case in generated(SCHEMA, |b| b.object_type_definition().map(Some))) {
            if let Some(doc) = &case.document {
                prop_assert_valid(doc)?;
            }
        }
    }
}
