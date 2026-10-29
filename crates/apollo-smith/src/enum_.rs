use crate::definition::TypeDefinition;
use crate::DocumentBuilder;
use apollo_compiler::ast;
use apollo_compiler::ast::DirectiveLocation;
use apollo_compiler::Name;
use apollo_compiler::Node;
use arbitrary::Result;
use indexmap::IndexSet;

impl DocumentBuilder<'_> {
    /// Generate an enum definition or extension.
    pub fn enum_type_definition(&mut self) -> Result<ast::Definition> {
        let extend = !self.enum_type_defs.is_empty() && self.u.arbitrary().unwrap_or(false);
        let description = self.optional_description()?;
        let name = if extend {
            let available_enums: Vec<&Name> = self
                .enum_type_defs
                .iter()
                .filter_map(|enm| {
                    if enm.is_extension_definition() {
                        None
                    } else {
                        Some(enm.type_name())
                    }
                })
                .collect();
            (*self.u.choose(&available_enums)?).clone()
        } else {
            self.type_name()?
        };
        let existing_values: IndexSet<Name> = self
            .enum_type_defs
            .iter()
            .filter(|e| e.type_name() == &name)
            .flat_map(|e| e.enum_values().iter().map(|v| v.value.clone()))
            .collect();
        let enum_values_def: Vec<Node<ast::EnumValueDefinition>> = self
            .enum_values_definition(&existing_values)?
            .into_iter()
            .map(|mut value| {
                self.drop_directives_reaching(&mut value.make_mut().directives, &name);
                value
            })
            .collect();
        let applied: IndexSet<Name> = self
            .enum_type_defs
            .iter()
            .filter(|def| def.type_name() == &name)
            .flat_map(|def| {
                def.directives()
                    .iter()
                    .map(|directive| directive.name.clone())
            })
            .collect();
        let mut directives = self.type_directives(DirectiveLocation::Enum, &applied)?;
        self.drop_directives_reaching(&mut directives, &name);

        if extend && directives.is_empty() && enum_values_def.is_empty() {
            return Err(arbitrary::Error::IncorrectFormat);
        }

        Ok(if extend {
            ast::EnumTypeExtension {
                name,
                directives,
                values: enum_values_def,
            }
            .into()
        } else {
            ast::EnumTypeDefinition {
                description,
                name,
                directives,
                values: enum_values_def,
            }
            .into()
        })
    }

    /// Choose an existing enum definition or extension.
    pub fn choose_enum(&mut self) -> Result<&ast::Definition> {
        self.u.choose(&self.enum_type_defs)
    }

    /// Create an arbitrary variant `Name` given an enum
    pub fn arbitrary_variant<'b>(&mut self, enum_: &'b ast::Definition) -> Result<&'b Name> {
        let arbitrary_idx = self.u.int_in_range(0..=(enum_.enum_values().len() - 1))?;
        Ok(enum_
            .enum_values()
            .get(arbitrary_idx)
            .map(|e| &e.value)
            .expect("cannot get variant"))
    }

    /// An arbitrary value of the enum at `enum_index`, borrowing the enum
    /// rather than cloning it to choose from.
    pub(crate) fn arbitrary_value_of_enum(&mut self, enum_index: usize) -> Result<Name> {
        let values = self.enum_type_defs[enum_index].enum_values();
        Ok(self.u.choose(values)?.value.clone())
    }

    /// Create an arbitrary `EnumValueDefinition`
    pub fn enum_values_definition(
        &mut self,
        exclude: &IndexSet<Name>,
    ) -> Result<Vec<Node<ast::EnumValueDefinition>>> {
        let mut enum_values_def =
            indexmap::IndexMap::with_capacity(self.u.int_in_range(2..=10usize)?);
        for i in 0..self.u.int_in_range(2..=10usize)? {
            let description = self.optional_description()?;
            let value = self.name_with_index(i)?;
            let directives = self.directives(DirectiveLocation::EnumValue)?;

            if !exclude.contains(&value) {
                enum_values_def.entry(value.clone()).or_insert_with(|| {
                    Node::new(ast::EnumValueDefinition {
                        description,
                        value,
                        directives,
                    })
                });
            }
        }

        Ok(enum_values_def.into_values().collect())
    }
}

#[cfg(test)]
mod tests {
    use crate::test_support::generated;
    use crate::test_support::prop_assert_valid;
    use proptest::prelude::*;

    const SCHEMA: &str = "schema { query: Query }
        directive @d(x: Int, y: [String!]!, f: F) on ENUM | ENUM_VALUE
        type Query { e: E }
        enum E @d(y: []) { A B }
        enum F { C }";

    proptest! {
        #[test]
        fn generates_valid_definitions(case in generated(SCHEMA, |b| b.enum_type_definition().map(Some))) {
            if let Some(doc) = &case.document {
                prop_assert_valid(doc)?;
            }
        }
    }
}
