use crate::next::mutations::Mutation;
use crate::next::unstructured::Unstructured;
use apollo_compiler::ast::Document;
use apollo_compiler::Node;

pub(crate) struct AddField;
impl Mutation for AddField {
    fn apply(&self, u: &mut Unstructured, doc: &mut Document) -> arbitrary::Result<()> {
        u.document(doc).with_object_type_definition(|u, o| {
            let mut field_definition = u.valid().field_definition()?;
            let existing_fields = o.fields.iter().map(|f| &f.name).collect::<Vec<_>>();
            field_definition.name = u.arbitrary_unique_name(&existing_fields)?;
            o.fields.push(Node::new(u.valid().field_definition()?));
            Ok(())
        })?;
        Ok(())
    }
    fn is_valid(&self) -> bool {
        true
    }
}
