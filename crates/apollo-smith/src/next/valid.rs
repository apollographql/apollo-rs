use crate::next::unstructured::Unstructured;
use apollo_compiler::ast::{FieldDefinition, Name, ObjectTypeDefinition};
use std::ops::{Deref, DerefMut};

pub(crate) struct Valid<'u, 'ue, 'n>(pub &'n mut Unstructured<'u, 'ue>);

impl<'u, 'ue, 'n> Deref for Valid<'u, 'ue, 'n> {
    type Target = Unstructured<'u, 'ue>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<'u, 'ue, 'n> DerefMut for Valid<'u, 'ue, 'n> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<'u, 'ue, 'n> Valid<'u, 'ue, 'n> {
    pub(crate) fn object_type_definition(&'n mut self) -> arbitrary::Result<ObjectTypeDefinition> {
        Ok(ObjectTypeDefinition {
            description: self.arbitrary_node_str()?.into(),
            name: self.valid().type_name()?,
            implements_interfaces: vec![],
            directives: Default::default(),
            fields: vec![self.valid().field_definition()?.into()],
        })
    }

    pub(crate) fn type_name(&mut self) -> arbitrary::Result<Name> {
        let existing_type_names = self.schema().types.keys().cloned().collect::<Vec<_>>();
        loop {
            let name = self.arbitrary_name()?;
            if !existing_type_names.contains(&name) {
                return Ok(name);
            }
        }
    }
    pub(crate) fn field_definition(&mut self) -> arbitrary::Result<FieldDefinition> {
        Ok(FieldDefinition {
            description: self.arbitrary_node_str()?.into(),
            name: self.arbitrary_name()?,
            arguments: vec![],
            ty: self.existing().ty()?,
            directives: Default::default(),
        })
    }
}
