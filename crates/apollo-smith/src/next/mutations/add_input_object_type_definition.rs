use apollo_compiler::ast::{Definition, Document};
use apollo_compiler::{Node, Schema};

use crate::next::mutations::SchemaMutation;
use crate::next::unstructured::Unstructured;

pub(crate) struct AddInputObjectTypeDefinition;
impl SchemaMutation for AddInputObjectTypeDefinition {
    fn apply(
        &self,
        u: &mut Unstructured,
        doc: &mut Document,
        schema: &Schema,
    ) -> arbitrary::Result<bool> {
        doc.definitions
            .push(Definition::InputObjectTypeDefinition(Node::new(
                u.arbitrary_input_object_type_definition(schema)?,
            )));
        Ok(true)
    }

    fn is_valid(&self) -> bool {
        true
    }
}
