use crate::next::mutations::Mutation;
use crate::next::unstructured::Unstructured;
use apollo_compiler::ast::Document;

pub(crate) struct AddObjectType;
impl Mutation for AddObjectType {
    fn apply(&self, u: &mut Unstructured, doc: &mut Document) -> arbitrary::Result<()> {
        doc.definitions
            .push(apollo_compiler::ast::Definition::ObjectTypeDefinition(
                u.valid().object_type_definition()?.into(),
            ));
        Ok(())
    }
    fn is_valid(&self) -> bool {
        true
    }
}
