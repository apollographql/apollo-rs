use apollo_compiler::ast::{Definition, Document};
use apollo_compiler::{Node, Schema};

use crate::next::mutations::Mutation;
use crate::next::unstructured::Unstructured;

pub(crate) struct AddDirectiveDefinition;
impl Mutation for AddDirectiveDefinition {
    fn apply(
        &self,
        u: &mut Unstructured,
        doc: &mut Document,
        schema: &Schema,
    ) -> arbitrary::Result<bool> {
        doc.definitions
            .push(Definition::DirectiveDefinition(Node::new(
                u.arbitrary_directive_definition(schema)?,
            )));

        Ok(true)
    }
    fn is_valid(&self) -> bool {
        true
    }
}
