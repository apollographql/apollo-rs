use crate::validation::Details;
use crate::validation::DiagnosticList;
use crate::validation::FileId;
use crate::InputDatabase;
use crate::Schema;
use std::sync::Arc;

pub(crate) fn validate_schema(errors: &mut DiagnosticList, schema: &Schema) {
    compiler_validation(errors, schema)
}

/// TODO: replace this with validation based on `Schema` without a database
fn compiler_validation(errors: &mut DiagnosticList, schema: &Schema) {
    let mut compiler = crate::ApolloCompiler::new();
    let mut ids = Vec::new();
    for (id, source) in schema.sources.iter() {
        ids.push(*id);
        compiler.db.set_input(*id, source.into());
    }
    compiler.db.set_schema(Arc::new(schema.clone()));
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
    for diagnostic in crate::validation::validate_type_system(&compiler.db) {
        errors.push(diagnostic.location, Details::CompilerDiagnostic(diagnostic))
    }
}
