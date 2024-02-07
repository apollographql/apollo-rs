use crate::next::mutations::Mutation;
use crate::next::unstructured::Unstructured;
use apollo_compiler::ast::Document;

pub(crate) struct RemoveAllFields;
impl Mutation for RemoveAllFields {
    fn apply(&self, u: &mut Unstructured, doc: &mut Document) -> arbitrary::Result<bool> {
        u.document(doc)
            .with_object_type_definition(|u, o| Ok(o.fields.clear()))?;
        Ok(true)
    }
    fn is_valid(&self) -> bool {
        false
    }
}
