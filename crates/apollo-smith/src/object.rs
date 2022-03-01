use std::collections::HashSet;

use apollo_encoder::ObjectDefinition;
use arbitrary::Result;

use crate::{
    description::Description, directive::Directive, field::FieldDef, name::Name, DocumentBuilder,
};

/// Object types represent concrete instantiations of sets of fields.
///
/// The introspection types (e.g. `__Type`, `__Field`, etc) are examples of
/// objects.
///
/// *ObjectTypeDefinition*:
///     Description? **type** Name ImplementsInterfaces? Directives? FieldsDefinition?
///
/// Detailed documentation can be found in [GraphQL spec](https://spec.graphql.org/October2021/#sec-Object).
#[derive(Debug, Clone)]
pub struct ObjectTypeDef {
    pub(crate) description: Option<Description>,
    pub(crate) name: Name,
    pub(crate) interface_impls: HashSet<Name>,
    pub(crate) directives: Vec<Directive>,
    pub(crate) fields_def: Vec<FieldDef>,
    pub(crate) extend: bool,
}

impl From<ObjectTypeDef> for ObjectDefinition {
    fn from(val: ObjectTypeDef) -> Self {
        let mut object_def = ObjectDefinition::new(val.name.into());
        val.interface_impls
            .into_iter()
            .for_each(|itf| object_def.interface(itf.into()));
        val.fields_def
            .into_iter()
            .for_each(|fd| object_def.field(fd.into()));
        object_def.description(val.description.map(String::from));
        val.directives
            .into_iter()
            .for_each(|directive| object_def.directive(directive.into()));
        if val.extend {
            object_def.extend();
        }

        object_def
    }
}

impl From<apollo_parser::ast::ObjectTypeDefinition> for ObjectTypeDef {
    fn from(object_def: apollo_parser::ast::ObjectTypeDefinition) -> Self {
        Self {
            name: object_def
                .name()
                .expect("object type definition must have a name")
                .into(),
            description: object_def.description().map(Description::from),
            // TODO
            directives: Vec::new(),
            // TODO
            interface_impls: HashSet::new(),
            extend: false,
            fields_def: object_def
                .fields_definition()
                .expect("object type definition must have fields definition")
                .field_definitions()
                .map(FieldDef::from)
                .collect(),
        }
    }
}

impl<'a> DocumentBuilder<'a> {
    /// Create an arbitrary `ObjectTypeDef`
    pub fn object_type_definition(&mut self) -> Result<ObjectTypeDef> {
        let description = self
            .u
            .arbitrary()
            .unwrap_or(false)
            .then(|| self.description())
            .transpose()?;
        let name = self.type_name()?;

        // ---- Interface
        let interface_impls = self.implements_interfaces()?;
        let implements_fields: Vec<FieldDef> = interface_impls
            .iter()
            .flat_map(|itf_name| {
                self.interface_type_defs
                    .iter()
                    .find(|itf| &itf.name == itf_name)
                    .expect("cannot find the corresponding interface")
                    .fields_def
                    .clone()
            })
            .collect();

        let mut fields_def = self.fields_definition(
            &implements_fields
                .iter()
                .map(|f| &f.name)
                .collect::<Vec<&Name>>(),
        )?;
        // Add fields coming from interfaces
        fields_def.extend(implements_fields);

        Ok(ObjectTypeDef {
            description,
            directives: self.directives()?,
            interface_impls,
            name,
            fields_def,
            extend: self.u.arbitrary().unwrap_or(false),
        })
    }
}
