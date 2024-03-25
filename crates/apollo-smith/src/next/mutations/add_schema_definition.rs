use apollo_compiler::ast::{Definition, Document};
use apollo_compiler::{Node, Schema};

use crate::next::mutations::SchemaMutation;
use crate::next::unstructured::Unstructured;

pub(crate) struct AddSchemaDefiniton;
impl SchemaMutation for AddSchemaDefiniton {
    fn apply(
        &self,
        u: &mut Unstructured,
        doc: &mut Document,
        schema: &Schema,
    ) -> arbitrary::Result<bool> {
        // If the document already has a schema definition, we don't need to add another one
        doc.definitions.push(Definition::SchemaDefinition(Node::new(
            u.arbitrary_schema_definition(schema)?,
        )));

        Ok(true)
    }
    fn is_valid(&self) -> bool {
        true
    }
}
