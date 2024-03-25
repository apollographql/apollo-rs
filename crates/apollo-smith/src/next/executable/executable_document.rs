use crate::next::Unstructured;
use apollo_compiler::executable::Fragment;
use apollo_compiler::{ExecutableDocument, Node};
use crate::next::schema::Selectable;

pub(crate) trait ExecutableDocumentExt {
    fn random_fragment(&self, u: &mut Unstructured) -> arbitrary::Result<&Node<Fragment>> {
        let fragments = self.target().fragments.values().collect::<Vec<_>>();
        Ok(fragments[u.choose_index(fragments.len())?])
    }

    fn random_fragment_of_type(
        &self,
        u: &mut Unstructured,
        selectable: &impl Selectable
    ) -> arbitrary::Result<Option<&Node<Fragment>>> {
        let fragments = self.target().fragments.values().filter(|f|&f.selection_set.ty == selectable.name()).collect::<Vec<_>>();
        if fragments.is_empty() {
            return Ok(None)
        }
        Ok(Some(fragments[u.choose_index(fragments.len())?]))
    }
    fn target(&self) -> &ExecutableDocument;
}

impl ExecutableDocumentExt for ExecutableDocument {
    fn target(&self) -> &ExecutableDocument {
        &self
    }
}
