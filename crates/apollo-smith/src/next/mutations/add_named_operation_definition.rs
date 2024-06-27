use crate::next::mutations::{ExecutableDocumentMutation};

use crate::next::unstructured::Unstructured;
use apollo_compiler::ast::{Definition, Document};
use apollo_compiler::{ExecutableDocument, Node, Schema};

pub(crate) struct AddNamedOperationDefinition;

impl ExecutableDocumentMutation for AddNamedOperationDefinition {
    fn apply(
        &self,
        u: &mut Unstructured,
        doc: &mut Document,
        schema: &Schema,
        executable_document: &ExecutableDocument,
    ) -> arbitrary::Result<bool> {
        if executable_document.anonymous_operation.is_some() {
            // We already have an anonymous operation, so we can't add a named one
            return Ok(false);
        }
        let name = u.unique_name();
        doc.definitions
            .push(Definition::OperationDefinition(Node::new(
                u.arbitrary_operation_definition(schema, executable_document, Some(name))?,
            )));
        Ok(true)
    }
    fn is_valid(&self) -> bool {
        true
    }
}
