use crate::next::mutations::SchemaMutation;

use crate::next::ast::definition::DefinitionKind;
use crate::next::ast::document::DocumentExt;
use crate::next::schema::InterfaceTypeExt;
use crate::next::unstructured::Unstructured;
use apollo_compiler::ast::{Definition, Document};
use apollo_compiler::Schema;

pub(crate) struct RemoveRequiredField;

impl SchemaMutation for RemoveRequiredField {
    fn apply(
        &self,
        u: &mut Unstructured,
        doc: &mut Document,
        schema: &Schema,
    ) -> arbitrary::Result<bool> {
        match doc.random_definition_mut(
            u,
            vec![
                DefinitionKind::ObjectTypeDefinition,
                DefinitionKind::InterfaceTypeDefinition,
            ],
        )? {
            Some(Definition::ObjectTypeDefinition(definition)) => {
                if !definition.implements_interfaces.is_empty() {
                    let interface = schema
                        .get_interface(u.choose(&definition.implements_interfaces)?)
                        .expect("interface not found");
                    if let Some(field) = interface.random_field(u)? {
                        definition
                            .make_mut()
                            .fields
                            .retain(|f| f.name != field.name);
                        return Ok(true);
                    }
                }
                Ok(false)
            }
            Some(Definition::InterfaceTypeDefinition(definition)) => {
                if !definition.implements_interfaces.is_empty() {
                    let interface = schema
                        .get_interface(u.choose(&definition.implements_interfaces)?)
                        .expect("interface not found");
                    if let Some(field) = interface.random_field(u)? {
                        definition
                            .make_mut()
                            .fields
                            .retain(|f| f.name != field.name);
                        return Ok(true);
                    }
                }
                Ok(false)
            }

            _ => Ok(false),
        }
    }
    fn is_valid(&self) -> bool {
        false
    }
}
