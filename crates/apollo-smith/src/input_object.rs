use std::collections::HashMap;

use arbitrary::Result;

use crate::{
    description::Description,
    directive::{Directive, DirectiveLocation},
    input_value::InputValueDef,
    name::Name,
    DocumentBuilder,
};

/// Input objects are composite types used as inputs into queries defined as a list of named input values..
///
/// InputObjectTypeDefinition
///     Description? **input** Name Directives? FieldsDefinition?
///
/// Detailed documentation can be found in [GraphQL spec](https://spec.graphql.org/October2021/#sec-Input-Objects).
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
    pub(crate) directives: HashMap<Name, Directive>,
    pub(crate) extend: bool,
}

impl From<InputObjectTypeDef> for apollo_encoder::InputObjectDefinition {
    fn from(input_object_def: InputObjectTypeDef) -> Self {
        let mut new_input_object_def = Self::new(input_object_def.name.into());
        if let Some(description) = input_object_def.description {
            new_input_object_def.description(description.into());
        }
        if input_object_def.extend {
            new_input_object_def.extend();
        }

        input_object_def
            .directives
            .into_iter()
            .for_each(|(_, directive)| new_input_object_def.directive(directive.into()));
        input_object_def
            .fields
            .into_iter()
            .for_each(|field| new_input_object_def.field(field.into()));

        new_input_object_def
    }
}

#[cfg(feature = "parser-impl")]
impl TryFrom<apollo_parser::ast::InputObjectTypeDefinition> for InputObjectTypeDef {
    type Error = crate::FromError;

    fn try_from(
        input_object: apollo_parser::ast::InputObjectTypeDefinition,
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

#[cfg(feature = "parser-impl")]
impl TryFrom<apollo_parser::ast::InputObjectTypeExtension> for InputObjectTypeDef {
    type Error = crate::FromError;

    fn try_from(
        input_object: apollo_parser::ast::InputObjectTypeExtension,
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

impl<'a> DocumentBuilder<'a> {
    /// Create an arbitrary `InputObjectTypeDef`
    pub fn input_object_type_definition(&mut self) -> Result<InputObjectTypeDef> {
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
        let fields = self.input_values_def()?;

        Ok(InputObjectTypeDef {
            description,
            directives: self.directives(DirectiveLocation::InputObject)?,
            name,
            extend,
            fields,
        })
    }
}
