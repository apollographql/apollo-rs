use arbitrary::Unstructured;

use apollo_compiler::ast::ObjectTypeDefinition;
use apollo_compiler::Node;

pub(crate) trait ObjectTypeDefinitionExt {
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
