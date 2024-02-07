use crate::next::mutations::Mutation;
use crate::next::unstructured::Unstructured;
use apollo_compiler::ast::Document;

pub(crate) struct RemoveAllFields;
impl Mutation for RemoveAllFields {
    fn apply(&self, u: &mut Unstructured, doc: &mut Document) -> arbitrary::Result<()> {
        u.document(doc)
            .with_object_type_definition(|_u, o| Ok(o.fields.clear()))?;
        Ok(())
    }
    fn is_valid(&self) -> bool {
        false
    }
}
