use std::collections::HashSet;

use apollo_encoder::ObjectDef;
use arbitrary::Result;

use crate::{
    description::Description, directive::Directive, field::FieldDef, name::Name, DocumentBuilder,
};

#[derive(Debug)]
pub struct ObjectTypeDef {
    pub(crate) description: Option<Description>,
    pub(crate) name: Name,
    pub(crate) interface_impls: HashSet<Name>,
    pub(crate) directives: Vec<Directive>,
    pub(crate) fields_def: Vec<FieldDef>,
    pub(crate) extend: bool,
}

impl From<ObjectTypeDef> for ObjectDef {
    fn from(val: ObjectTypeDef) -> Self {
        let mut object_def = ObjectDef::new(val.name.into());
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
        let interface_impls = self.interface_implements()?;
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
