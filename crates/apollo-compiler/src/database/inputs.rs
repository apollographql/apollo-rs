use super::sources::{FileId, Source, SourceType};
use std::sync::Arc;

#[salsa::query_group(InputStorage)]
pub(crate) trait InputDatabase {
    /// Get input source of the corresponding file.
    #[salsa::input]
    fn input(&self, file_id: FileId) -> Source;

    #[salsa::input]
    fn schema(&self) -> Arc<crate::Schema>;

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

fn source_type(db: &dyn InputDatabase, file_id: FileId) -> SourceType {
    db.input(file_id).source_type()
}

fn type_definition_files(db: &dyn InputDatabase) -> Vec<FileId> {
    db.source_files()
        .into_iter()
        .filter(|source| matches!(db.source_type(*source), SourceType::Schema))
        .collect()
}

fn executable_definition_files(db: &dyn InputDatabase) -> Vec<FileId> {
    db.source_files()
        .into_iter()
        .filter(|source| matches!(db.source_type(*source), SourceType::Executable))
        .collect()
}
