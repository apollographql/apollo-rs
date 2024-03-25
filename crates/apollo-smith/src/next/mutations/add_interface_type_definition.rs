use apollo_compiler::ast::{Definition, Document};
use apollo_compiler::{Node, Schema};

use crate::next::mutations::SchemaMutation;
use crate::next::unstructured::Unstructured;

pub(crate) struct AddInterfaceTypeDefinition;
impl SchemaMutation for AddInterfaceTypeDefinition {
    fn apply(
        &self,
        u: &mut Unstructured,
        doc: &mut Document,
        schema: &Schema,
    ) -> arbitrary::Result<bool> {
        doc.definitions
            .push(Definition::InterfaceTypeDefinition(Node::new(
                u.arbitrary_interface_type_definition(schema)?,
            )));

        Ok(true)
    }
    fn is_valid(&self) -> bool {
        true
    }
}
