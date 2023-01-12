use super::sources::{FileId, Source, SourceType};
use crate::hir::TypeSystem;
use std::collections::HashMap;
use ariadne::Source as AriadneSource;
use std::sync::Arc;

#[salsa::query_group(InputStorage)]
pub trait InputDatabase {
    /// Get the currently set recursion limit.
    #[salsa::input]
    fn recursion_limit(&self) -> Option<usize>;

    /// Get input source of the corresponding file.
    #[salsa::input]
    fn type_system_hir_input(&self) -> Option<Arc<TypeSystem>>;

    #[salsa::input]
    fn input(&self, file_id: FileId) -> Source;

    /// Get the GraphQL source text for a file.
    fn source_code(&self, file_id: FileId) -> Arc<str>;

    /// Get the source type (document/schema/executable) for a file.
    fn source_type(&self, file_id: FileId) -> SourceType;

    /// Get all file ids currently in the compiler.
    #[salsa::input]
    fn source_files(&self) -> Vec<FileId>;

    fn source_with_lines(&self, file_id: FileId) -> Arc<AriadneSource>;
    fn source_cache(&self) -> Arc<HashMap<FileId, Arc<AriadneSource>>>;

    /// Get all type system definition (GraphQL schema) files.
    fn type_definition_files(&self) -> Vec<FileId>;

    /// Get all executable definition (GraphQL query) files.
    fn executable_definition_files(&self) -> Vec<FileId>;
}

fn source_code(db: &dyn InputDatabase, file_id: FileId) -> Arc<str> {
    // For diagnostics, also include sources for a precomputed input.
    if let Some(precomputed) = db.type_system_hir_input() {
        if let Some(source) = precomputed.inputs.get(&file_id) {
            return source.text();
        }
    }
    db.input(file_id).text()
}

fn source_type(db: &dyn InputDatabase, file_id: FileId) -> SourceType {
    db.input(file_id).source_type()
}

fn source_with_lines(db: &dyn InputDatabase, file_id: FileId) -> Arc<AriadneSource> {
    let code = db.source_code(file_id);
    Arc::new(AriadneSource::from(code))
}

fn source_cache(db: &dyn InputDatabase) -> Arc<HashMap<FileId, Arc<AriadneSource>>> {
    let map = db.source_files()
        .into_iter()
        .map(|id| {
            (id, db.source_with_lines(id))
        })
        .collect();
    Arc::new(map)
}

fn type_definition_files(db: &dyn InputDatabase) -> Vec<FileId> {
    db.source_files()
        .into_iter()
        .filter(|source| {
            matches!(
                db.source_type(*source),
                SourceType::Schema | SourceType::Document
            )
        })
        .collect()
}

fn executable_definition_files(db: &dyn InputDatabase) -> Vec<FileId> {
    db.source_files()
        .into_iter()
        .filter(|source| {
            matches!(
                db.source_type(*source),
                SourceType::Query | SourceType::Document
            )
        })
        .collect()
}
