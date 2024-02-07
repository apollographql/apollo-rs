use crate::next::unstructured::Unstructured;
use apollo_compiler::ast::{Name, Type};
use std::ops::{Deref, DerefMut};

pub(crate) struct Existing<'u, 'ue, 'e>(pub(crate) &'e mut Unstructured<'u, 'ue>);
impl<'u, 'ue, 'e> Existing<'u, 'ue, 'e> {
    pub(crate) fn type_name(&mut self) -> arbitrary::Result<Name> {
        let names = self.schema().types.keys().cloned().collect::<Vec<_>>();
        assert!(!names.is_empty());
        Ok(self.choose(&names)?.clone())
    }

    pub(crate) fn ty(&mut self) -> arbitrary::Result<Type> {
        let name = self.type_name()?;
        self.wrap_ty(name)
    }
}

impl<'u, 'ue, 'n> Deref for Existing<'u, 'ue, 'n> {
    type Target = Unstructured<'u, 'ue>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'u, 'ue, 'n> DerefMut for Existing<'u, 'ue, 'n> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
