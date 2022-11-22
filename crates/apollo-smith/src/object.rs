use std::collections::{HashMap, HashSet};

use apollo_encoder::ObjectDefinition;
use arbitrary::Result;

use crate::{
    description::Description,
    directive::{Directive, DirectiveLocation},
    field::FieldDef,
    name::Name,
    DocumentBuilder, StackedEntity,
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
    pub(crate) implements_interfaces: HashSet<Name>,
    pub(crate) directives: HashMap<Name, Directive>,
    pub(crate) fields_def: Vec<FieldDef>,
    pub(crate) extend: bool,
}

impl From<ObjectTypeDef> for ObjectDefinition {
    fn from(val: ObjectTypeDef) -> Self {
        let mut object_def = ObjectDefinition::new(val.name.into());
        if let Some(description) = val.description {
            object_def.description(description.into())
        }
        val.implements_interfaces
            .into_iter()
            .for_each(|itf| object_def.interface(itf.into()));
        val.fields_def
            .into_iter()
            .for_each(|fd| object_def.field(fd.into()));

        val.directives
            .into_iter()
            .for_each(|(_, directive)| object_def.directive(directive.into()));
        if val.extend {
            object_def.extend();
        }

        object_def
    }
}

#[cfg(feature = "parser-impl")]
impl TryFrom<apollo_parser::ast::ObjectTypeDefinition> for ObjectTypeDef {
    type Error = crate::FromError;

    fn try_from(object_def: apollo_parser::ast::ObjectTypeDefinition) -> Result<Self, Self::Error> {
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

#[cfg(feature = "parser-impl")]
impl TryFrom<apollo_parser::ast::ObjectTypeExtension> for ObjectTypeDef {
    type Error = crate::FromError;

    fn try_from(object_def: apollo_parser::ast::ObjectTypeExtension) -> Result<Self, Self::Error> {
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

impl<'a> DocumentBuilder<'a> {
    /// Create an arbitrary `ObjectTypeDef`
    pub fn object_type_definition(&mut self) -> Result<ObjectTypeDef> {
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
            directives: self.directives(DirectiveLocation::Object)?,
            implements_interfaces: interface_impls,
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
