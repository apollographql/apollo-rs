use crate::next::mutations::{ExecutableDocumentMutation};

use crate::next::unstructured::Unstructured;
use apollo_compiler::ast::{Definition, Document};
use apollo_compiler::{ExecutableDocument, Node, Schema};

pub(crate) struct AddAnonymousOperationDefinition;

impl ExecutableDocumentMutation for AddAnonymousOperationDefinition {
    fn apply(
        &self,
        u: &mut Unstructured,
        doc: &mut Document,
        schema: &Schema,
        executable_document: &ExecutableDocument,
    ) -> arbitrary::Result<bool> {
        if !executable_document.named_operations.is_empty() || executable_document.anonymous_operation.is_some() {
            // We already have an operation, so we can't add an anonymous one
            return Ok(false);
        }
        doc.definitions
            .push(Definition::OperationDefinition(Node::new(
                u.arbitrary_operation_definition(schema, executable_document, None)?,
            )));
        Ok(true)
    }
    fn is_valid(&self) -> bool {
        true
    }
}
