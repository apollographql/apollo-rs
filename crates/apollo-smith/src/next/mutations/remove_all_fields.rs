use crate::next::mutations::SchemaMutation;

use crate::next::ast::definition::DefinitionKind;
use crate::next::ast::document::DocumentExt;
use crate::next::unstructured::Unstructured;
use apollo_compiler::ast::{Definition, Document};
use apollo_compiler::Schema;

pub(crate) struct RemoveAllFields;
impl SchemaMutation for RemoveAllFields {
    fn apply(
        &self,
        u: &mut Unstructured,
        doc: &mut Document,
        _schema: &Schema,
    ) -> arbitrary::Result<bool> {
        match doc.random_definition_mut(
            u,
            vec![
                DefinitionKind::ObjectTypeDefinition,
                DefinitionKind::InterfaceTypeDefinition,
            ],
        )? {
            Some(Definition::ObjectTypeDefinition(definition)) => {
                definition.make_mut().fields.clear();
                Ok(true)
            }
            Some(Definition::InterfaceTypeDefinition(definition)) => {
                definition.make_mut().fields.clear();
                Ok(true)
            }

            _ => Ok(false),
        }
    }
    fn is_valid(&self) -> bool {
        false
    }
}
