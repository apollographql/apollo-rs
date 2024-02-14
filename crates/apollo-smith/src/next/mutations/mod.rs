use arbitrary::Unstructured;
use std::any::type_name;

use crate::next::mutations::add_directive_definition::AddDirectiveDefinition;
use crate::next::mutations::add_input_object_type_definition::AddInputObjectTypeDefinition;
use crate::next::mutations::add_interface_type_definition::AddInterfaceTypeDefinition;
use apollo_compiler::ast::Document;
use apollo_compiler::Schema;

use crate::next::mutations::add_object_type_definition::AddObjectTypeDefinition;
use crate::next::mutations::add_schema_definition::AddSchemaDefiniton;
use crate::next::mutations::add_union_type_definition::AddUnionTypeDefinition;

mod add_directive_definition;
mod add_enum_type_definition;
mod add_input_object_type_definition;
mod add_interface_type_definition;
mod add_object_type_definition;
mod add_schema_definition;
mod add_union_type_definition;
mod remove_all_fields;

pub(crate) trait Mutation {
    fn apply(
        &self,
        u: &mut Unstructured,
        doc: &mut Document,
        schema: &Schema,
    ) -> arbitrary::Result<()>;
    fn is_valid(&self) -> bool;

    fn type_name(&self) -> &str {
        type_name::<Self>()
    }
}

pub(crate) fn all_mutations() -> Vec<Box<dyn Mutation>> {
    vec![
        Box::new(AddObjectTypeDefinition),
        Box::new(AddInterfaceTypeDefinition),
        Box::new(AddDirectiveDefinition),
        Box::new(AddInputObjectTypeDefinition),
        Box::new(AddUnionTypeDefinition),
        Box::new(AddInterfaceTypeDefinition),
    ]
}
