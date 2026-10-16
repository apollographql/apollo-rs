use crate::description::Description;
use crate::directive::applied_directives;
use crate::directive::Directive;
use crate::directive::DirectiveLocation;
use crate::input_value::InputValueDef;
use crate::name::Name;
use crate::DocumentBuilder;
use apollo_compiler::ast;
use apollo_compiler::Node;
use arbitrary::Result as ArbitraryResult;
use indexmap::IndexMap;
use indexmap::IndexSet;

/// Input objects are composite types used as inputs into queries defined as a list of named input values..
///
/// InputObjectTypeDefinition
///     Description? **input** Name Directives? FieldsDefinition?
///
/// Detailed documentation can be found in [GraphQL spec](https://spec.graphql.org/September2025/#sec-Input-Objects).
///
/// **Note**: At the moment InputObjectTypeDefinition differs slightly from the
/// spec. Instead of accepting InputValues as `field` parameter, we accept
/// InputField.
#[derive(Debug, Clone)]
pub struct InputObjectTypeDef {
    pub(crate) name: Name,
    pub(crate) description: Option<Description>,
    // A vector of fields
    pub(crate) fields: Vec<InputValueDef>,
    /// Contains all directives.
    pub(crate) directives: IndexMap<Name, Directive>,
    pub(crate) extend: bool,
}

impl From<InputObjectTypeDef> for ast::Definition {
    fn from(x: InputObjectTypeDef) -> Self {
        if x.extend {
            ast::InputObjectTypeExtension {
                name: x.name.into(),
                directives: Directive::to_ast(x.directives),
                fields: x.fields.into_iter().map(|x| Node::new(x.into())).collect(),
            }
            .into()
        } else {
            ast::InputObjectTypeDefinition {
                description: x.description.map(Into::into),
                name: x.name.into(),
                directives: Directive::to_ast(x.directives),
                fields: x.fields.into_iter().map(|x| Node::new(x.into())).collect(),
            }
            .into()
        }
    }
}

impl TryFrom<apollo_parser::cst::InputObjectTypeDefinition> for InputObjectTypeDef {
    type Error = crate::FromError;

    fn try_from(
        input_object: apollo_parser::cst::InputObjectTypeDefinition,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            name: input_object
                .name()
                .expect("object type definition must have a name")
                .into(),
            description: input_object.description().map(Description::from),
            directives: input_object
                .directives()
                .map(Directive::convert_directives)
                .transpose()?
                .unwrap_or_default(),
            extend: false,
            fields: input_object
                .input_fields_definition()
                .map(|input_fields| {
                    input_fields
                        .input_value_definitions()
                        .map(InputValueDef::try_from)
                        .collect::<Result<_, _>>()
                })
                .transpose()?
                .unwrap_or_default(),
        })
    }
}

impl TryFrom<apollo_parser::cst::InputObjectTypeExtension> for InputObjectTypeDef {
    type Error = crate::FromError;

    fn try_from(
        input_object: apollo_parser::cst::InputObjectTypeExtension,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            name: input_object
                .name()
                .expect("object type definition must have a name")
                .into(),
            directives: input_object
                .directives()
                .map(Directive::convert_directives)
                .transpose()?
                .unwrap_or_default(),
            extend: true,
            fields: input_object
                .input_fields_definition()
                .map(|input_fields| {
                    input_fields
                        .input_value_definitions()
                        .map(InputValueDef::try_from)
                        .collect::<Result<Vec<_>, crate::FromError>>()
                })
                .transpose()?
                .unwrap_or_default(),
            description: None,
        })
    }
}

impl DocumentBuilder<'_> {
    pub(crate) fn is_one_of_input(&self, name: &Name) -> bool {
        self.input_object_type_defs
            .iter()
            .any(|io| &io.name == name && io.directives.keys().any(|d| d.name == "oneOf"))
    }

    /// Create an arbitrary `InputObjectTypeDef`
    pub fn input_object_type_definition(&mut self) -> ArbitraryResult<InputObjectTypeDef> {
        let extend = !self.input_object_type_defs.is_empty() && self.u.arbitrary().unwrap_or(false);
        let name = if extend {
            let available_input_objects: Vec<&Name> = self
                .input_object_type_defs
                .iter()
                .filter_map(|input_object| {
                    if input_object.extend {
                        None
                    } else {
                        Some(&input_object.name)
                    }
                })
                .collect();
            (*self.u.choose(&available_input_objects)?).clone()
        } else {
            self.type_name()?
        };
        let description = self
            .u
            .arbitrary()
            .unwrap_or(false)
            .then(|| self.description())
            .transpose()?;
        let exclude_fields: IndexSet<Name> = self
            .input_object_type_defs
            .iter()
            .filter(|io| io.name == name)
            .flat_map(|io| io.fields.iter().map(|f| f.name.clone()))
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
                field.ty = field.ty.clone().into_nullable();
            }
            if is_one_of || extends_one_of {
                field.default_value = None;
            }
        }

        let applied = applied_directives(
            self.input_object_type_defs
                .iter()
                .filter(|def| def.name == name)
                .map(|def| &def.directives),
        );
        let mut directives = self.type_directives(DirectiveLocation::InputObject, &applied)?;
        if is_one_of {
            directives.insert(
                Name::new(String::from("oneOf")),
                Directive {
                    name: Name::new(String::from("oneOf")),
                    arguments: Vec::new(),
                },
            );
        }

        if extend && directives.is_empty() && fields.is_empty() {
            return Err(arbitrary::Error::IncorrectFormat);
        }

        Ok(InputObjectTypeDef {
            description,
            directives,
            name,
            extend,
            fields,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::test_support::generate;
    use crate::test_support::input;
    use crate::test_support::prop_assert_valid;
    use proptest::prelude::*;

    // Extensions of these must keep existing values valid and follow
    // @oneOf's rules.
    const SCHEMA: &str = "schema { query: Query }
        directive @d(x: Int, y: [String!]!) on INPUT_OBJECT | INPUT_FIELD_DEFINITION
        type Query { f(i: In, one: One): Int }
        input In @d(y: []) { a: Int, b: String! }
        input Other { in: In, c: [Int] }
        input One @oneOf { a: Int, b: String }";

    proptest! {
        #[test]
        fn generates_valid_definitions(data in input()) {
            if let Some(doc) = generate(SCHEMA, &data, |b| b.input_object_type_definition().map(Some)) {
                prop_assert_valid(&doc)?;
            }
        }
    }
}
