use crate::validation::diagnostics::ValidationError;
use crate::validation::directive::validate_directive_definitions;
use crate::validation::enum_::validate_enum_definitions;
use crate::validation::input_object::validate_input_object_definitions;
use crate::validation::interface::validate_interface_definitions;
use crate::validation::object::validate_object_type_definitions;
use crate::validation::scalar::validate_scalar_definitions;
use crate::validation::schema::validate_schema_definition;
use crate::validation::union_::validate_union_definitions;
use crate::{ExecutableDocument, Schema};

pub(crate) fn validate_type_system(schema: &crate::Schema) -> Vec<ValidationError> {
    let mut diagnostics = Vec::new();

    diagnostics.extend(validate_schema_definition(schema));

    diagnostics.extend(validate_scalar_definitions(schema));
    diagnostics.extend(validate_enum_definitions(schema));
    diagnostics.extend(validate_union_definitions(schema));

    diagnostics.extend(validate_interface_definitions(schema));
    diagnostics.extend(validate_directive_definitions(schema));
    diagnostics.extend(validate_input_object_definitions(schema));
    diagnostics.extend(validate_object_type_definitions(schema));

    diagnostics
}

pub(crate) fn validate_executable(
    document: &ExecutableDocument,
    schema: Option<&Schema>,
) -> Vec<ValidationError> {
    let mut diagnostics = Vec::new();

    diagnostics.extend(super::operation::validate_operation_definitions(
        schema, document,
    ));
    for def in document.fragments.values() {
        diagnostics.extend(super::fragment::validate_fragment_used(document, def));
    }

    diagnostics
}
