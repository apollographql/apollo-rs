use std::any::type_name;

use apollo_compiler::ast::Document;
use apollo_compiler::{ExecutableDocument, Schema};

use crate::next::mutations::add_anonymous_operation_definition::AddAnonymousOperationDefinition;
use crate::next::mutations::add_directive_definition::AddDirectiveDefinition;
use crate::next::mutations::add_enum_type_definition::AddEnumTypeDefinition;
use crate::next::mutations::add_input_object_type_definition::AddInputObjectTypeDefinition;
use crate::next::mutations::add_interface_type_definition::AddInterfaceTypeDefinition;
use crate::next::mutations::add_named_operation_definition::AddNamedOperationDefinition;
use crate::next::mutations::add_object_type_definition::AddObjectTypeDefinition;
use crate::next::mutations::add_union_type_definition::AddUnionTypeDefinition;
use crate::next::mutations::remove_all_fields::RemoveAllFields;
use crate::next::mutations::remove_required_field::RemoveRequiredField;
use crate::next::unstructured::Unstructured;

mod add_anonymous_operation_definition;
mod add_directive_definition;
mod add_enum_type_definition;
mod add_fragment_definiton;
mod add_input_object_type_definition;
mod add_interface_type_definition;
mod add_named_operation_definition;
mod add_object_type_definition;
mod add_schema_definition;
mod add_union_type_definition;
mod remove_all_fields;
mod remove_required_field;

pub(crate) trait SchemaMutation {
    /// Apply the mutation to the document
    /// Returns false if the mutation did not apply
    fn apply(
        &self,
        u: &mut Unstructured,
        doc: &mut Document,
        schema: &Schema,
    ) -> arbitrary::Result<bool>;
    fn is_valid(&self) -> bool;

    fn type_name(&self) -> &str {
        type_name::<Self>()
    }
}

pub(crate) trait ExecutableDocumentMutation {
    /// Apply the mutation to the document
    /// Returns false if the mutation did not apply
    fn apply(
        &self,
        u: &mut Unstructured,
        doc: &mut Document,
        schema: &Schema,
        executable: &ExecutableDocument,
    ) -> arbitrary::Result<bool>;
    fn is_valid(&self) -> bool;

    fn type_name(&self) -> &str {
        type_name::<Self>()
    }
}

pub(crate) fn schema_mutations() -> Vec<Box<dyn SchemaMutation>> {
    vec![
        Box::new(AddObjectTypeDefinition),
        Box::new(AddInterfaceTypeDefinition),
        Box::new(AddDirectiveDefinition),
        Box::new(AddInputObjectTypeDefinition),
        Box::new(AddUnionTypeDefinition),
        Box::new(AddEnumTypeDefinition),
        //Box::new(RemoveAllFields),
        Box::new(RemoveRequiredField),
    ]
}

pub(crate) fn executable_document_mutations() -> Vec<Box<dyn ExecutableDocumentMutation>> {
    vec![
        Box::new(AddNamedOperationDefinition),
        Box::new(AddAnonymousOperationDefinition),
    ]
}
