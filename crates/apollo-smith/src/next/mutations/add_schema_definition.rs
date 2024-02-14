use arbitrary::Unstructured;

use apollo_compiler::ast::{Definition, Document};
use apollo_compiler::{Node, Schema};

use crate::next::ast::document::DocumentExt;
use crate::next::mutations::Mutation;

pub(crate) struct AddSchemaDefiniton;
impl Mutation for AddSchemaDefiniton {
    fn apply(
        &self,
        u: &mut Unstructured,
        doc: &mut Document,
        schema: &Schema,
    ) -> arbitrary::Result<()> {
        // If the document already has a schema definition, we don't need to add another one
        doc.definitions.push(Definition::SchemaDefinition(Node::new(
            doc.arbitrary_schema_definition(u, schema)?,
        )));

        Ok(())
    }
    fn is_valid(&self) -> bool {
        true
    }
}
