mod add_field;
mod add_object_type_definition;
mod remove_all_fields;

use crate::next::mutations::add_object_type_definition::AddObjectType;
use crate::next::unstructured::Unstructured;
use apollo_compiler::ast::Document;

pub(crate) trait Mutation {
    fn apply(&self, u: &mut Unstructured, doc: &mut Document) -> arbitrary::Result<bool>;
    fn is_valid(&self) -> bool;
}

pub(crate) fn all_mutations() -> Vec<Box<dyn Mutation>> {
    vec![Box::new(AddObjectType)]
}
