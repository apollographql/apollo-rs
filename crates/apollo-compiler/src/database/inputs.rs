use super::sources::{FileId, Source, SourceType};
use std::sync::Arc;

#[salsa::query_group(InputStorage)]
pub(crate) trait InputDatabase {
    /// Get input source of the corresponding file.
    #[salsa::input]
    fn input(&self, file_id: FileId) -> Source;

    #[salsa::input]
    fn recursion_limit(&self) -> usize;

    #[salsa::input]
    fn schema_input(&self) -> Option<Arc<crate::Schema>>;

    /// Get the GraphQL source text for a file.
    #[salsa::invoke(source_code)]
    fn source_code(&self, file_id: FileId) -> Arc<String>;

    /// Get the source type (document/schema/executable) for a file.
    #[salsa::invoke(source_type)]
    fn source_type(&self, file_id: FileId) -> SourceType;

    /// Get all file ids currently in the compiler.
    #[salsa::input]
    fn source_files(&self) -> Vec<FileId>;

    /// Get all type system definition (GraphQL schema) files.
    #[salsa::invoke(type_definition_files)]
    fn type_definition_files(&self) -> Vec<FileId>;

    /// Get all executable definition (GraphQL query) files.
    #[salsa::invoke(executable_definition_files)]
    fn executable_definition_files(&self) -> Vec<FileId>;
}

fn source_code(db: &dyn InputDatabase, file_id: FileId) -> Arc<String> {
    db.input(file_id).text().clone()
}

fn source_type(db: &dyn InputDatabase, file_id: FileId) -> SourceType {
    db.input(file_id).source_type()
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
                SourceType::Executable | SourceType::Document
            )
        })
        .collect()
}
