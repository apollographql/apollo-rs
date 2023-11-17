use super::BuildError;
use crate::validation::Details;
use crate::validation::DiagnosticList;
use crate::validation::FileId;
use crate::InputDatabase;
use crate::Schema;
use crate::ValidationDatabase;
use std::sync::Arc;

pub(crate) fn validate_schema(errors: &mut DiagnosticList, schema: &Schema) {
    for (&file_id, source) in schema.sources.iter() {
        source.validate_parse_errors(errors, file_id)
    }
    for build_error in &schema.build_errors {
        validate_build_error(errors, build_error)
    }
    compiler_validation(errors, schema)
}

fn validate_build_error(errors: &mut DiagnosticList, build_error: &BuildError) {
    match build_error {
        BuildError::ExecutableDefinition { location, .. }
        | BuildError::SchemaDefinitionCollision { location, .. }
        | BuildError::DirectiveDefinitionCollision { location, .. }
        | BuildError::TypeDefinitionCollision { location, .. }
        | BuildError::BuiltInScalarTypeRedefinition { location, .. }
        | BuildError::OrphanSchemaExtension { location, .. }
        | BuildError::OrphanTypeExtension { location, .. }
        | BuildError::TypeExtensionKindMismatch { location, .. }
        | BuildError::DuplicateRootOperation { location, .. }
        | BuildError::DuplicateImplementsInterfaceInObject { location, .. }
        | BuildError::DuplicateImplementsInterfaceInInterface { location, .. }
        | BuildError::ObjectFieldNameCollision { location, .. }
        | BuildError::InterfaceFieldNameCollision { location, .. }
        | BuildError::EnumValueNameCollision { location, .. }
        | BuildError::UnionMemberNameCollision { location, .. }
        | BuildError::InputFieldNameCollision { location, .. } => {
            errors.push(*location, Details::SchemaBuildError(build_error.clone()))
        }
    }
}

/// TODO: replace this with validation based on `Schema` without a database
fn compiler_validation(errors: &mut DiagnosticList, schema: &Schema) {
    let mut compiler = crate::ApolloCompiler::new();
    let mut ids = Vec::new();
    for (id, source) in schema.sources.iter() {
        ids.push(*id);
        compiler.db.set_input(*id, source.into());
    }
    let ast_id = FileId::HACK_TMP;
    ids.push(ast_id);
    let mut ast = crate::ast::Document::new();
    ast.definitions.extend(schema.to_ast());
    compiler.db.set_input(
        ast_id,
        crate::Source {
            ty: crate::database::SourceType::Schema,
            filename: Default::default(),
            text: Default::default(),
            ast: Some(Arc::new(ast)),
        },
    );
    compiler.db.set_source_files(ids);
    for diagnostic in compiler.db.validate_type_system() {
        errors.push(diagnostic.location, Details::CompilerDiagnostic(diagnostic))
    }
}
