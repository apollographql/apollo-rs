use apollo_compiler::ast::{Definition, Document};
use apollo_compiler::{Node, Schema};

use crate::next::ast::document::DocumentExt;
use crate::next::mutations::Mutation;
use crate::next::unstructured::Unstructured;

pub(crate) struct AddObjectTypeDefinition;
impl Mutation for AddObjectTypeDefinition {
    fn apply(
        &self,
        u: &mut Unstructured,
        doc: &mut Document,
        schema: &Schema,
    ) -> arbitrary::Result<()> {
        doc.definitions
            .push(Definition::ObjectTypeDefinition(Node::new(
                u.arbitrary_object_type_definition(schema)?,
            )));
        Ok(())
    }

    fn is_valid(&self) -> bool {
        true
    }
}
