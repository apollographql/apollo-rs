use crate::next::mutations::{ExecutableDocumentMutation};

use crate::next::unstructured::Unstructured;
use apollo_compiler::ast::{Definition, Document};
use apollo_compiler::{ExecutableDocument, Node, Schema};

pub(crate) struct AddFragmentDefiniton;

impl ExecutableDocumentMutation for AddFragmentDefiniton {
    fn apply(
        &self,
        u: &mut Unstructured,
        doc: &mut Document,
        schema: &Schema,
        executable_document: &ExecutableDocument,
    ) -> arbitrary::Result<bool> {
        doc.definitions
            .push(Definition::FragmentDefinition(Node::new(
                u.arbitrary_fragment_definition(schema, executable_document)?,
            )));
        Ok(true)
    }
    fn is_valid(&self) -> bool {
        true
    }
}
