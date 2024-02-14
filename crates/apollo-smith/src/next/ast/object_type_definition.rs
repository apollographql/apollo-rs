use arbitrary::Unstructured;

use apollo_compiler::ast::ObjectTypeDefinition;
use apollo_compiler::Node;

use crate::next::common::Common;

pub(crate) trait ObjectTypeDefinitionExt: Common {
    field_access!();

    fn target(&self) -> &ObjectTypeDefinition;
    fn target_mut(&mut self) -> &mut ObjectTypeDefinition;
}

impl ObjectTypeDefinitionExt for ObjectTypeDefinition {
    fn target(&self) -> &ObjectTypeDefinition {
        self
    }
    fn target_mut(&mut self) -> &mut ObjectTypeDefinition {
        self
    }
}

impl Common for ObjectTypeDefinition {}
