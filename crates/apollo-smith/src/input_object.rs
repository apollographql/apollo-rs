use arbitrary::Result;

use crate::{
    description::Description, directive::Directive, input_value::InputValueDef, name::Name,
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
    pub(crate) directives: Vec<Directive>,
    pub(crate) extend: bool,
}

impl From<InputObjectTypeDef> for apollo_encoder::InputObjectDefinition {
    fn from(input_object_def: InputObjectTypeDef) -> Self {
        let mut new_input_object_def = Self::new(input_object_def.name.into());
        new_input_object_def.description(input_object_def.description.map(String::from));
        if input_object_def.extend {
            new_input_object_def.extend();
        }

        input_object_def
            .directives
            .into_iter()
            .for_each(|directive| new_input_object_def.directive(directive.into()));
        input_object_def
            .fields
            .into_iter()
            .for_each(|field| new_input_object_def.field(field.into()));

        new_input_object_def
    }
}

impl<'a> DocumentBuilder<'a> {
    /// Create an arbitrary `InputObjectTypeDef`
    pub fn input_object_type_definition(&mut self) -> Result<InputObjectTypeDef> {
        let description = self
            .u
            .arbitrary()
            .unwrap_or(false)
            .then(|| self.description())
            .transpose()?;
        let name = self.type_name()?;
        let fields = self.input_values_def()?;

        Ok(InputObjectTypeDef {
            description,
            directives: self.directives()?,
            name,
            extend: self.u.arbitrary().unwrap_or(false),
            fields,
        })
    }
}
