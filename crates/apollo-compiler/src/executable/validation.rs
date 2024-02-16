use super::FieldSet;
use crate::ast;
use crate::database::RootDatabase;
use crate::validation::selection::FieldsInSetCanMerge;
use crate::validation::Details;
use crate::validation::DiagnosticList;
use crate::validation::FileId;
use crate::validation::Valid;
use crate::ExecutableDocument;
use crate::InputDatabase;
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
    let mut ids = Vec::new();

    if let Some(schema) = schema {
        db.set_schema(Arc::new(schema.clone()));
    }

    let ast_id = FileId::HACK_TMP;
    ids.push(ast_id);
    let ast = document.to_ast();
    db.set_input(
        ast_id,
        crate::Source {
            ty: crate::database::SourceType::Executable,
            ast: Some(Arc::new(ast)),
        },
    );
    db.set_source_files(ids);
    let diagnostics = if schema.is_some() {
        crate::validation::validate_executable(&db, ast_id)
    } else {
        crate::validation::validate_standalone_executable(&db, ast_id)
    };
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
    let mut ids = Vec::new();

    db.set_schema(Arc::new(schema.as_ref().clone()));

    let ast_id = FileId::HACK_TMP;
    ids.push(ast_id);
    let ast = ast::Document::new();
    db.set_input(
        ast_id,
        crate::Source {
            ty: crate::database::SourceType::Executable,
            ast: Some(Arc::new(ast)),
        },
    );
    db.set_source_files(ids);
    let diagnostics = crate::validation::selection::validate_selection_set(
        &db,
        ast_id,
        Some(&field_set.selection_set.ty),
        &field_set.selection_set.to_ast(),
        crate::validation::operation::OperationValidationConfig {
            has_schema: true,
            variables: &[],
        },
    );
    //  if schema.is_some() {
    //     compiler.db.validate_executable(ast_id)
    // } else {
    //     compiler.db.validate_standalone_executable(ast_id)
    // };
    for diagnostic in diagnostics {
        errors.push(diagnostic.location, Details::CompilerDiagnostic(diagnostic))
    }
}
