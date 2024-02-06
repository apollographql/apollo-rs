use crate::next::existing::Existing;
use crate::next::invalid::Invalid;
use crate::next::valid::Valid;
use apollo_compiler::ast::Name;
use apollo_compiler::NodeStr;
use arbitrary::Result;
use std::ops::{Deref, DerefMut};

pub(crate) struct Unstructured<'u, 'ue> {
    pub(crate) u: &'ue mut arbitrary::Unstructured<'u>,
    pub(crate) schema: &'ue apollo_compiler::Schema,
}

impl Unstructured<'_, '_> {
    pub(crate) fn new<'u, 'ue>(
        u: &'ue mut arbitrary::Unstructured<'u>,
        schema: &'ue apollo_compiler::Schema,
    ) -> Unstructured<'u, 'ue> {
        Unstructured { u, schema }
    }
}

impl<'u, 'ue> Deref for Unstructured<'u, 'ue> {
    type Target = arbitrary::Unstructured<'u>;

    fn deref(&self) -> &Self::Target {
        &self.u
    }
}

impl<'u, 'ue> DerefMut for Unstructured<'u, 'ue> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.u
    }
}

impl<'u, 'ue> Unstructured<'u, 'ue> {
    pub(crate) fn valid(&mut self) -> Valid<'u, 'ue, '_> {
        Valid(self)
    }

    pub(crate) fn existing(&mut self) -> Existing<'u, 'ue, '_> {
        Existing(self)
    }

    pub(crate) fn invalid(&mut self) -> Invalid<'u, 'ue, '_> {
        Invalid(self)
    }
    pub(crate) fn schema(&self) -> &apollo_compiler::Schema {
        &self.schema
    }

    pub(crate) fn arbitrary_node_str(&mut self) -> Result<NodeStr> {
        Ok(NodeStr::new(self.arbitrary()?))
    }

    pub(crate) fn arbitrary_name(&mut self) -> Result<Name> {
        loop {
            if let Ok(name) = Name::new(self.arbitrary_node_str()?) {
                return Ok(name);
            }
        }
    }
}
