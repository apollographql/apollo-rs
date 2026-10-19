use crate::description::Description;
use crate::directive::Directive;
use crate::directive::DirectiveLocation;
use crate::field::FieldDef;
use crate::interface::field_signatures_for;
use crate::name::Name;
use crate::DocumentBuilder;
use crate::StackedEntity;
use apollo_compiler::ast;
use apollo_compiler::Node;
use arbitrary::Result as ArbitraryResult;
use indexmap::IndexMap;
use indexmap::IndexSet;

/// Object types represent concrete instantiations of sets of fields.
///
/// The introspection types (e.g. `__Type`, `__Field`, etc) are examples of
/// objects.
///
/// *ObjectTypeDefinition*:
///     Description? **type** Name ImplementsInterfaces? Directives? FieldsDefinition?
///
/// Detailed documentation can be found in [GraphQL spec](https://spec.graphql.org/September2025/#sec-Objects).
#[derive(Debug, Clone)]
pub struct ObjectTypeDef {
    pub(crate) description: Option<Description>,
    pub(crate) name: Name,
    pub(crate) implements_interfaces: IndexSet<Name>,
    pub(crate) directives: IndexMap<Name, Directive>,
    pub(crate) fields_def: Vec<FieldDef>,
    pub(crate) extend: bool,
}

impl From<ObjectTypeDef> for ast::Definition {
    fn from(x: ObjectTypeDef) -> Self {
        if x.extend {
            ast::ObjectTypeExtension {
                name: x.name.into(),
                implements_interfaces: x
                    .implements_interfaces
                    .into_iter()
                    .map(Into::into)
                    .collect(),
                directives: Directive::to_ast(x.directives),
                fields: x
                    .fields_def
                    .into_iter()
                    .map(|x| Node::new(x.into()))
                    .collect(),
            }
            .into()
        } else {
            ast::ObjectTypeDefinition {
                description: x.description.map(Into::into),
                name: x.name.into(),
                implements_interfaces: x
                    .implements_interfaces
                    .into_iter()
                    .map(Into::into)
                    .collect(),
                directives: Directive::to_ast(x.directives),
                fields: x
                    .fields_def
                    .into_iter()
                    .map(|x| Node::new(x.into()))
                    .collect(),
            }
            .into()
        }
    }
}

impl TryFrom<apollo_parser::cst::ObjectTypeDefinition> for ObjectTypeDef {
    type Error = crate::FromError;

    fn try_from(object_def: apollo_parser::cst::ObjectTypeDefinition) -> Result<Self, Self::Error> {
        Ok(Self {
            name: object_def
                .name()
                .expect("object type definition must have a name")
                .into(),
            description: object_def.description().map(Description::from),
            directives: object_def
                .directives()
                .map(Directive::convert_directives)
                .transpose()?
                .unwrap_or_default(),
            implements_interfaces: object_def
                .implements_interfaces()
                .map(|impl_int| {
                    impl_int
                        .named_types()
                        .map(|n| n.name().unwrap().into())
                        .collect()
                })
                .unwrap_or_default(),
            extend: false,
            fields_def: object_def
                .fields_definition()
                .expect("object type definition must have fields definition")
                .field_definitions()
                .map(FieldDef::try_from)
                .collect::<Result<Vec<_>, _>>()?,
        })
    }
}

impl TryFrom<apollo_parser::cst::ObjectTypeExtension> for ObjectTypeDef {
    type Error = crate::FromError;

    fn try_from(object_def: apollo_parser::cst::ObjectTypeExtension) -> Result<Self, Self::Error> {
        Ok(Self {
            name: object_def
                .name()
                .expect("object type definition must have a name")
                .into(),
            description: None,
            directives: object_def
                .directives()
                .map(Directive::convert_directives)
                .transpose()?
                .unwrap_or_default(),
            implements_interfaces: object_def
                .implements_interfaces()
                .map(|impl_int| {
                    impl_int
                        .named_types()
                        .map(|n| n.name().unwrap().into())
                        .collect()
                })
                .unwrap_or_default(),
            extend: true,
            fields_def: object_def
                .fields_definition()
                .expect("object type definition must have fields definition")
                .field_definitions()
                .map(FieldDef::try_from)
                .collect::<Result<Vec<_>, _>>()?,
        })
    }
}

impl DocumentBuilder<'_> {
    /// Create an arbitrary `ObjectTypeDef`
    pub fn object_type_definition(&mut self) -> ArbitraryResult<ObjectTypeDef> {
        let extend = !self.object_type_defs.is_empty() && self.u.arbitrary().unwrap_or(false);
        let description = self
            .u
            .arbitrary()
            .unwrap_or(false)
            .then(|| self.description())
            .transpose()?;
        let name = if extend {
            let available_objects: Vec<&Name> = self
                .object_type_defs
                .iter()
                .filter_map(|object| {
                    if object.extend {
                        None
                    } else {
                        Some(&object.name)
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
            .filter(|def| def.name == name)
            .flat_map(|def| def.directives.keys().cloned())
            .collect();
        let directives = self.type_directives(DirectiveLocation::Object, &applied)?;

        if extend
            && directives.is_empty()
            && fields_def.is_empty()
            && implements_interfaces.is_empty()
        {
            return Err(arbitrary::Error::IncorrectFormat);
        }

        Ok(ObjectTypeDef {
            description,
            directives,
            implements_interfaces,
            name,
            fields_def,
            extend,
        })
    }
}

impl StackedEntity for ObjectTypeDef {
    fn name(&self) -> &Name {
        &self.name
    }

    fn fields_def(&self) -> &[FieldDef] {
        &self.fields_def
    }
}

#[cfg(test)]
mod tests {
    use crate::test_support::generate;
    use crate::test_support::input;
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
        fn generates_valid_definitions(data in input()) {
            if let Some(doc) = generate(SCHEMA, &data, |b| b.object_type_definition().map(Some)) {
                prop_assert_valid(&doc)?;
            }
        }
    }
}
