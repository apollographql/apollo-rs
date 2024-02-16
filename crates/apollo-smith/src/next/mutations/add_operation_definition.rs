use crate::next::mutations::Mutation;

use crate::next::unstructured::Unstructured;
use apollo_compiler::ast::{Definition, Document};
use apollo_compiler::{Node, Schema};

pub(crate) struct AddOperationDefinition;

impl Mutation for AddOperationDefinition {
    fn apply(
        &self,
        u: &mut Unstructured,
        doc: &mut Document,
        schema: &Schema,
    ) -> arbitrary::Result<bool> {
        doc.definitions
            .push(Definition::OperationDefinition(Node::new(
                u.arbitrary_operation_definition(schema)?,
            )));
        Ok(false)
    }
    fn is_valid(&self) -> bool {
        false
    }
}
