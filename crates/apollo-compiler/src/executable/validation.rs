use super::FieldSet;
use crate::validation::fragment::validate_fragment_used;
use crate::validation::operation::validate_operation_definitions;
use crate::validation::selection::FieldsInSetCanMerge;
use crate::validation::DiagnosticList;
use crate::validation::ExecutableValidationContext;
use crate::validation::Valid;
use crate::ExecutableDocument;
use crate::Schema;

pub(crate) fn validate_executable_document(
    errors: &mut DiagnosticList,
    schema: &Schema,
    document: &ExecutableDocument,
) {
    validate_with_or_without_schema(errors, Some(schema), document);
    validate_with_schema(errors, schema, document);
}

pub(crate) fn validate_standalone_executable(
    errors: &mut DiagnosticList,
    document: &ExecutableDocument,
) {
    validate_with_or_without_schema(errors, None, document);
}

fn validate_with_schema(
    errors: &mut DiagnosticList,
    schema: &Schema,
    document: &ExecutableDocument,
) {
    let alloc = typed_arena::Arena::new();
    let mut fields_in_set_can_merge = FieldsInSetCanMerge::new(&alloc, schema, document);
    for operation in document.all_operations() {
        crate::validation::operation::validate_subscription(document, operation, errors);
        fields_in_set_can_merge.validate_operation(operation, errors);
    }
}

pub(crate) fn validate_with_or_without_schema(
    errors: &mut DiagnosticList,
    schema: Option<&Schema>,
    document: &ExecutableDocument,
) {
    let context = ExecutableValidationContext::new(schema);
    validate_operation_definitions(errors, document, &context);
    for def in document.fragments.values() {
        validate_fragment_used(errors, document, def);
    }
}

pub(crate) fn validate_field_set(
    diagnostics: &mut DiagnosticList,
    schema: &Valid<Schema>,
    field_set: &FieldSet,
) {
    let document = &ExecutableDocument::new(); // No fragment definitions
    let context = ExecutableValidationContext::new(Some(schema));
    crate::validation::selection::validate_selection_set(
        diagnostics,
        document,
        Some((schema, &field_set.selection_set.ty)),
        &field_set.selection_set,
        context.operation_context(&[]),
    )
}
