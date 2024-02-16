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

pub(crate) use crate::ReprDatabase as ValidationDatabase;

pub(crate) fn validate_type_system(db: &dyn ValidationDatabase) -> Vec<ValidationError> {
    let mut diagnostics = Vec::new();

    diagnostics.extend(validate_schema_definition(db));

    diagnostics.extend(validate_scalar_definitions(db));
    diagnostics.extend(validate_enum_definitions(db));
    diagnostics.extend(validate_union_definitions(db));

    diagnostics.extend(validate_interface_definitions(db));
    diagnostics.extend(validate_directive_definitions(db));
    diagnostics.extend(validate_input_object_definitions(db));
    diagnostics.extend(validate_object_type_definitions(db));

    diagnostics
}

pub(crate) fn validate_executable(
    db: &dyn ValidationDatabase,
    document: &ExecutableDocument,
    schema: Option<&Schema>,
) -> Vec<ValidationError> {
    let mut diagnostics = Vec::new();

    diagnostics.extend(super::operation::validate_operation_definitions(
        db, document, schema,
    ));
    for def in document.fragments.values() {
        diagnostics.extend(super::fragment::validate_fragment_used(document, def));
    }

    diagnostics
}
