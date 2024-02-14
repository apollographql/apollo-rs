use crate::next::mutations::Mutation;
use arbitrary::Unstructured;

use crate::next::ast::document::DocumentExt;
use apollo_compiler::ast::Document;
use apollo_compiler::Schema;

pub(crate) struct RemoveAllFields;
impl Mutation for RemoveAllFields {
    fn apply(
        &self,
        u: &mut Unstructured,
        doc: &mut Document,
        schema: &Schema,
    ) -> arbitrary::Result<()> {
        doc.random_object_type_definition_mut(u)?
            .make_mut()
            .fields
            .clear();
        Ok(())
    }
    fn is_valid(&self) -> bool {
        false
    }
}
