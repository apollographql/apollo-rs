use super::FieldSet;
use crate::database::ReprDatabase;
use crate::database::RootDatabase;
use crate::validation::selection::FieldsInSetCanMerge;
use crate::validation::Details;
use crate::validation::DiagnosticList;
use crate::validation::Valid;
use crate::ExecutableDocument;
use crate::Schema;
use std::sync::Arc;

pub(crate) fn validate_executable_document(
    errors: &mut DiagnosticList,
    schema: &Schema,
    document: &ExecutableDocument,
) {
    validate_with_or_without_schema(errors, document);
    validate_with_schema(errors, schema, document);
    compiler_validation(errors, Some(schema), document);
    // TODO
}

pub(crate) fn validate_standalone_executable(
    errors: &mut DiagnosticList,
    document: &ExecutableDocument,
) {
    validate_with_or_without_schema(errors, document);
    compiler_validation(errors, None, document);
}

fn validate_with_schema(
    errors: &mut DiagnosticList,
    schema: &Schema,
    document: &ExecutableDocument,
) {
    let mut fields_in_set_can_merge = FieldsInSetCanMerge::new(schema, document);
    for operation in document.all_operations() {
        crate::validation::operation::validate_subscription(document, operation, errors);
        fields_in_set_can_merge.validate_operation(operation, errors);
    }
}

pub(crate) fn validate_with_or_without_schema(
    _errors: &mut DiagnosticList,
    _document: &ExecutableDocument,
) {
    // TODO
}

/// TODO: replace this with validation based on `ExecutableDocument` without a database
fn compiler_validation(
    errors: &mut DiagnosticList,
    schema: Option<&Schema>,
    document: &ExecutableDocument,
) {
    let mut db = RootDatabase::default();

    if let Some(schema) = schema {
        db.set_schema(Arc::new(schema.clone()));
    }

    let diagnostics = crate::validation::validate_executable(&db, document, schema);
    for diagnostic in diagnostics {
        errors.push(diagnostic.location, Details::CompilerDiagnostic(diagnostic))
    }
}

pub(crate) fn validate_field_set(
    errors: &mut DiagnosticList,
    schema: &Valid<Schema>,
    field_set: &FieldSet,
) {
    let mut db = RootDatabase::default();

    db.set_schema(Arc::new(schema.as_ref().clone()));

    let document = &ExecutableDocument::new(); // No fragment definitions
    let diagnostics = crate::validation::selection::validate_selection_set(
        &db,
        document,
        Some(&field_set.selection_set.ty),
        &field_set.selection_set,
        crate::validation::operation::OperationValidationConfig {
            has_schema: true,
            variables: &[],
        },
    );
    for diagnostic in diagnostics {
        errors.push(diagnostic.location, Details::CompilerDiagnostic(diagnostic))
    }
}
