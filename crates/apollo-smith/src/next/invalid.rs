use crate::next::unstructured::Unstructured;
use std::ops::{Deref, DerefMut};

pub(crate) struct Invalid<'u, 'ue, 'e>(pub(crate) &'e mut Unstructured<'u, 'ue>);
impl<'u, 'ue, 'e> Invalid<'u, 'ue, 'e> {}

impl<'u, 'ue, 'n> Deref for Invalid<'u, 'ue, 'n> {
    type Target = Unstructured<'u, 'ue>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'u, 'ue, 'n> DerefMut for Invalid<'u, 'ue, 'n> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
