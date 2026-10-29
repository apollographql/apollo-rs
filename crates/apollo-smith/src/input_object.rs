use crate::definition::TypeDefinition;
use crate::DocumentBuilder;
use apollo_compiler::ast;
use apollo_compiler::ast::Directive;
use apollo_compiler::ast::DirectiveLocation;
use apollo_compiler::name;
use apollo_compiler::Name;
use apollo_compiler::Node;
use arbitrary::Result as ArbitraryResult;
use indexmap::IndexSet;

impl DocumentBuilder<'_> {
    pub(crate) fn is_one_of_input(&self, name: &Name) -> bool {
        self.input_object_type_defs
            .iter()
            .any(|io| io.type_name() == name && io.directives().iter().any(|d| d.name == "oneOf"))
    }

    /// Generate an input object definition or extension.
    pub fn input_object_type_definition(&mut self) -> ArbitraryResult<ast::Definition> {
        let extend = !self.input_object_type_defs.is_empty() && self.u.arbitrary().unwrap_or(false);
        let name = if extend {
            let available_input_objects: Vec<&Name> = self
                .input_object_type_defs
                .iter()
                .filter_map(|input_object| {
                    if input_object.is_extension_definition() {
                        None
                    } else {
                        Some(input_object.type_name())
                    }
                })
                .collect();
            (*self.u.choose(&available_input_objects)?).clone()
        } else {
            self.type_name()?
        };
        let description = self.optional_description()?;
        let exclude_fields: IndexSet<Name> = self
            .input_object_type_defs
            .iter()
            .filter(|io| io.type_name() == &name)
            .flat_map(|io| io.input_fields().iter().map(|f| f.name.clone()))
            .collect();

        // Randomly apply @oneOf to a base definition (~1-in-5 chance). When
        // we do, enforce the spec constraints on every field: all must be
        // nullable and none may carry a default value. Extensions may not
        // add @oneOf, but must follow its constraints when the base has it.
        let is_one_of = !extend && self.u.ratio(1, 5)?;
        let extends_one_of = extend && self.is_one_of_input(&name);
        let mut fields = self.input_values_def(
            DirectiveLocation::InputFieldDefinition,
            &exclude_fields,
            Some(&name),
        )?;
        for field in &mut fields {
            // Values of this type may already have been generated from
            // earlier definitions, so extensions only add nullable fields.
            if is_one_of || extend {
                field.ty = Node::new((*field.ty).clone().nullable());
            }
            if is_one_of || extends_one_of {
                field.default_value = None;
            }
            self.drop_directives_reaching(&mut field.directives, &name);
        }

        let applied: IndexSet<Name> = self
            .input_object_type_defs
            .iter()
            .filter(|def| def.type_name() == &name)
            .flat_map(|def| {
                def.directives()
                    .iter()
                    .map(|directive| directive.name.clone())
            })
            .collect();
        let mut directives = self.type_directives(DirectiveLocation::InputObject, &applied)?;
        self.drop_directives_reaching(&mut directives, &name);
        if is_one_of {
            directives.push(Node::new(Directive {
                name: name!(oneOf),
                arguments: Vec::new(),
            }));
        }

        if extend && directives.is_empty() && fields.is_empty() {
            return Err(arbitrary::Error::IncorrectFormat);
        }

        Ok(if extend {
            ast::InputObjectTypeExtension {
                name,
                directives,
                fields: fields.into_iter().map(Node::new).collect(),
            }
            .into()
        } else {
            ast::InputObjectTypeDefinition {
                description,
                name,
                directives,
                fields: fields.into_iter().map(Node::new).collect(),
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

    // Extensions of these must keep existing values valid and follow
    // @oneOf's rules.
    const SCHEMA: &str = "schema { query: Query }
        directive @d(x: Int, y: [String!]!, one: One) on INPUT_OBJECT | INPUT_FIELD_DEFINITION
        type Query { f(i: In, one: One): Int }
        input In @d(y: []) { a: Int, b: String! }
        input Other { in: In, c: [Int] }
        input One @oneOf { a: Int, b: String }";

    proptest! {
        #[test]
        fn generates_valid_definitions(case in generated(SCHEMA, |b| b.input_object_type_definition().map(Some))) {
            if let Some(doc) = &case.document {
                prop_assert_valid(doc)?;
            }
        }
    }
}
