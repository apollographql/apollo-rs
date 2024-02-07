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
        let idx = self.int_in_range(0..=3)?;
        Ok(match idx {
            0 => Type::Named(self.existing().type_name()?.clone()),
            1 => Type::NonNullNamed(self.existing().type_name()?.clone()),
            2 => Type::List(Box::new(self.existing().ty()?)),
            _ => Type::NonNullList(Box::new(self.existing().ty()?)),
        })
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
