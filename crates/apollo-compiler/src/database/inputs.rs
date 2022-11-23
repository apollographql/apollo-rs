use crate::database::sources::FileId;

use super::sources::{Source, SourceManifest};
use std::sync::Arc;

#[salsa::query_group(InputStorage)]
pub trait InputDatabase {
    #[salsa::input]
    fn recursion_limit(&self) -> Option<usize>;

    #[salsa::input]
    fn input(&self, file_id: FileId) -> Source;

    fn source_code(&self, file_id: FileId) -> Arc<str>;

    // Arc?
    #[salsa::input]
    fn sources(&self) -> SourceManifest;

    // should we cache instead?
    /// Get all file ids currently in the compiler.
    #[salsa::transparent]
    fn source_files(&self) -> Vec<FileId>;
}

fn source_code(db: &dyn InputDatabase, file_id: FileId) -> Arc<str> {
    db.input(file_id).text()
}

fn source_files(db: &dyn InputDatabase) -> Vec<FileId> {
    db.sources().manifest.keys().copied().collect()
}
