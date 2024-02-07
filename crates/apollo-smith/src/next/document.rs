use crate::next::unstructured::Unstructured;
use apollo_compiler::ast::{FieldDefinition, ObjectTypeDefinition};
use std::ops::{Deref, DerefMut};

pub(crate) struct Document<'u, 'ue, 'd, 'ad> {
    pub(crate) u: &'d mut Unstructured<'u, 'ue>,
    pub(crate) doc: &'ad mut apollo_compiler::ast::Document,
}
impl<'u, 'ue, 'e, 'ad> Document<'u, 'ue, 'e, 'ad> {
    pub(crate) fn with_object_type_definition(
        &mut self,
        callback: fn(&mut Unstructured, ty: &mut ObjectTypeDefinition) -> arbitrary::Result<()>,
    ) -> arbitrary::Result<()> {
        let mut definitions = self
            .doc
            .definitions
            .iter_mut()
            .filter_map(|def| match def {
                apollo_compiler::ast::Definition::ObjectTypeDefinition(def) => Some(def.make_mut()),
                _ => None,
            })
            .collect::<Vec<_>>();

        let idx = self.u.choose_index(definitions.len())?;

        Ok(callback(self.u, definitions[idx])?)
    }

    pub(crate) fn with_field_definition(
        &mut self,
        callback: fn(&mut Unstructured, doc: &mut FieldDefinition) -> arbitrary::Result<()>,
    ) -> arbitrary::Result<()> {
        let mut definitions = self
            .doc
            .definitions
            .iter_mut()
            .filter_map(|def| match def {
                apollo_compiler::ast::Definition::ObjectTypeDefinition(def) => Some(def.make_mut()),
                _ => None,
            })
            .collect::<Vec<_>>();

        let idx = self.u.choose_index(definitions.len())?;

        Ok(callback(self.u, definitions[idx])?)
    }
}

impl<'u, 'ue, 'd, 'ad> Deref for Document<'u, 'ue, 'd, 'ad> {
    type Target = Unstructured<'u, 'ue>;

    fn deref(&self) -> &Self::Target {
        &self.u
    }
}

impl<'u, 'ue, 'd, 'ad> DerefMut for Document<'u, 'ue, 'd, 'ad> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.u
    }
}
