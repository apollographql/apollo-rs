use crate::database::sources::FileId;

use super::sources::{Source, SourceType, SourceManifest};
use std::sync::Arc;

#[salsa::query_group(InputStorage)]
pub trait InputDatabase {
    #[salsa::input]
    fn recursion_limit(&self) -> Option<usize>;

    #[salsa::input]
    fn input(&self, file_id: FileId) -> Source;

    /// Get the GraphQL source text for a file.
    fn source_code(&self, file_id: FileId) -> Arc<str>;

    fn source_type(&self, file_id: FileId) -> SourceType;

    // Arc?
    #[salsa::input]
    fn sources(&self) -> SourceManifest;

    // should we cache instead?
    /// Get all file ids currently in the compiler.
    #[salsa::transparent]
    fn source_files(&self) -> Vec<FileId>;

    fn type_definition_files(&self) -> Vec<FileId>;
    fn executable_definition_files(&self) -> Vec<FileId>;
}

fn source_code(db: &dyn InputDatabase, file_id: FileId) -> Arc<str> {
    db.input(file_id).text()
}

fn source_type(db: &dyn InputDatabase, file_id: FileId) -> SourceType {
    db.input(file_id).source_type()
}

fn source_files(db: &dyn InputDatabase) -> Vec<FileId> {
    db.sources().manifest.keys().copied().collect()
}

fn type_definition_files(db: &dyn InputDatabase) -> Vec<FileId> {
    db.source_files()
        .into_iter()
        .filter(|source| matches!(db.source_type(*source), SourceType::Schema | SourceType::Document))
        .collect()
}

fn executable_definition_files(db: &dyn InputDatabase) -> Vec<FileId> {
    db.source_files()
        .into_iter()
        .filter(|source| matches!(db.source_type(*source), SourceType::Query | SourceType::Document))
        .collect()
}
