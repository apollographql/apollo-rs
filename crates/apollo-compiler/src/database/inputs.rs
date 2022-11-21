use crate::database::sources::FileId;

use super::sources::SourceManifest;

#[salsa::query_group(InputStorage)]
pub trait InputDatabase {
    #[salsa::input]
    fn recursion_limit(&self) -> Option<usize>;

    #[salsa::input]
    fn input_document(&self, file_id: FileId) -> String;

    // NOTE(@lrlna): in the context of an environment where a schema represents
    // the current state that must not be modified from incoming queries, we
    // want to make sure there are two separate points of input: schemas and queries.
    #[salsa::input]
    fn input_schema(&self, file_id: FileId) -> String;

    #[salsa::input]
    fn input_query(&self, file_id: FileId) -> String;

    // Arc?
    #[salsa::input]
    fn sources(&self) -> SourceManifest;

    // should we cache instead?
    /// Get all file ids currently in the compiler.
    #[salsa::transparent]
    fn source_files(&self) -> Vec<FileId>;
}

fn source_files(db: &dyn InputDatabase) -> Vec<FileId> {
    db.sources().manifest.keys().copied().collect()
}
