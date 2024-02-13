use super::FieldSet;
use crate::ast;
use crate::validation::Details;
use crate::validation::DiagnosticList;
use crate::validation::FileId;
use crate::validation::Valid;
use crate::ExecutableDocument;
use crate::InputDatabase;
use crate::Schema;
use crate::ValidationDatabase;
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
    _errors: &mut DiagnosticList,
    _schema: &Schema,
    _document: &ExecutableDocument,
) {
    // TODO
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
    let mut compiler = crate::ApolloCompiler::new();
    let mut ids = Vec::new();
    if let Some(schema) = schema {
        for (id, source) in schema.sources.iter() {
            ids.push(*id);
            compiler.db.set_input(*id, source.into());
        }
    }
    for (id, source) in document.sources.iter() {
        ids.push(*id);
        compiler.db.set_input(*id, source.into());
    }

    if let Some(schema) = schema {
        compiler.db.set_schema_input(Some(Arc::new(schema.clone())));
    }

    let ast_id = FileId::HACK_TMP;
    ids.push(ast_id);
    let ast = document.to_ast();
    compiler.db.set_input(
        ast_id,
        crate::Source {
            ty: crate::database::SourceType::Executable,
            filename: Default::default(),
            text: Default::default(),
            ast: Some(Arc::new(ast)),
        },
    );
    compiler.db.set_source_files(ids);
    let diagnostics = if schema.is_some() {
        compiler.db.validate_executable(ast_id)
    } else {
        compiler.db.validate_standalone_executable(ast_id)
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
    let mut compiler = crate::ApolloCompiler::new();
    let mut ids = Vec::new();
    for (id, source) in &*schema.sources {
        ids.push(*id);
        compiler.db.set_input(*id, source.into());
    }
    for (id, source) in &*field_set.sources {
        ids.push(*id);
        compiler.db.set_input(*id, source.into());
    }

    compiler
        .db
        .set_schema_input(Some(Arc::new(schema.as_ref().clone())));

    let ast_id = FileId::HACK_TMP;
    ids.push(ast_id);
    let ast = ast::Document::new();
    compiler.db.set_input(
        ast_id,
        crate::Source {
            ty: crate::database::SourceType::Executable,
            filename: Default::default(),
            text: Default::default(),
            ast: Some(Arc::new(ast)),
        },
    );
    compiler.db.set_source_files(ids);
    let diagnostics = crate::validation::selection::validate_selection_set(
        &compiler.db,
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
