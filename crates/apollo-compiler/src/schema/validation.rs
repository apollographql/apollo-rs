use crate::database::RootDatabase;
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
    let mut db = RootDatabase::default();
    let mut ids = Vec::new();
    db.set_schema(Arc::new(schema.clone()));
    let ast_id = FileId::HACK_TMP;
    ids.push(ast_id);
    let mut ast = crate::ast::Document::new();
    ast.definitions.extend(schema.to_ast());
    db.set_input(
        ast_id,
        crate::Source {
            ty: crate::database::SourceType::Schema,
            ast: Some(Arc::new(ast)),
        },
    );
    db.set_source_files(ids);
    for diagnostic in crate::validation::validate_type_system(&db) {
        errors.push(diagnostic.location, Details::CompilerDiagnostic(diagnostic))
    }
}
