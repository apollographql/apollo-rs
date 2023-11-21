use super::BuildError;
use super::FieldSet;
use crate::ast;
use crate::validation::Details;
use crate::validation::DiagnosticList;
use crate::validation::ValidationOptions;
use crate::ExecutableDocument;
use crate::FileId;
use crate::InputDatabase;
use crate::Schema;
use crate::ValidationDatabase;
use std::sync::Arc;

pub(crate) fn validate_executable_document(
    errors: &mut DiagnosticList,
    schema: &Schema,
    document: &ExecutableDocument,
    options: ValidationOptions,
) {
    validate_common(errors, document);
    compiler_validation(errors, Some(schema), document, options);
    // TODO
}

pub(crate) fn validate_standalone_executable(
    errors: &mut DiagnosticList,
    document: &ExecutableDocument,
    options: ValidationOptions,
) {
    validate_common(errors, document);
    compiler_validation(errors, None, document, options);
}

pub(crate) fn validate_common(errors: &mut DiagnosticList, document: &ExecutableDocument) {
    for (file_id, source) in document.sources.iter() {
        source.validate_parse_errors(errors, *file_id)
    }
    for build_error in &document.build_errors {
        validate_build_error(errors, build_error)
    }
    if let Some(operation) = &document.anonymous_operation {
        if !document.named_operations.is_empty()
            || document
                .build_errors
                .iter()
                .any(|e| matches!(e, BuildError::AmbiguousAnonymousOperation { .. }))
        {
            let location = operation.location();
            // Not actually a build error from converting from AST,
            // but reuses the same message formatting
            errors.push(
                location,
                Details::ExecutableBuildError(BuildError::AmbiguousAnonymousOperation { location }),
            )
        }
    }
    // TODO
}

fn validate_build_error(errors: &mut DiagnosticList, build_error: &BuildError) {
    let location = match build_error {
        BuildError::TypeSystemDefinition { location, .. }
        | BuildError::AmbiguousAnonymousOperation { location }
        | BuildError::OperationNameCollision { location, .. }
        | BuildError::FragmentNameCollision { location, .. }
        | BuildError::UndefinedRootOperation { location, .. }
        | BuildError::UndefinedField { location, .. }
        | BuildError::UndefinedTypeInNamedFragmentTypeCondition { location, .. }
        | BuildError::UndefinedTypeInInlineFragmentTypeCondition { location, .. }
        | BuildError::SubselectionOnScalarType { location, .. }
        | BuildError::SubselectionOnEnumType { location, .. } => *location,
    };
    errors.push(location, Details::ExecutableBuildError(build_error.clone()))
}

/// TODO: replace this with validation based on `ExecutableDocument` without a database
fn compiler_validation(
    errors: &mut DiagnosticList,
    schema: Option<&Schema>,
    document: &ExecutableDocument,
    options: ValidationOptions,
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

    compiler.db.set_recursion_limit(options.recursion_limit);

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
    schema: &Schema,
    field_set: &FieldSet,
) {
    for (file_id, source) in &*field_set.sources {
        source.validate_parse_errors(errors, *file_id)
    }
    for build_error in &field_set.build_errors {
        validate_build_error(errors, build_error)
    }
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

    compiler.db.set_schema_input(Some(Arc::new(schema.clone())));

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
    let diagnostics = crate::validation::selection::validate_selection_set2(
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
