use super::sources::{FileId, Source, SourceType};
use crate::Arc;
use ariadne::{Cache as AriadneCache, Source as AriadneSource};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
#[error("Unknown file ID")]
struct UnknownFileError;

/// A Cache implementation for `ariadne` diagnostics.
///
/// Use [`InputDatabase::source_cache`] to construct one.
#[derive(Clone, PartialEq, Eq)]
pub struct SourceCache {
    sources: HashMap<FileId, Arc<AriadneSource>>,
    paths: HashMap<FileId, PathBuf>,
}
impl AriadneCache<FileId> for &SourceCache {
    fn fetch(&mut self, id: &FileId) -> Result<&AriadneSource, Box<dyn std::fmt::Debug>> {
        let source = self.sources.get(id);
        source
            .map(|arc| &**arc)
            .ok_or_else(|| Box::new(UnknownFileError) as Box<dyn std::fmt::Debug>)
    }
    fn display<'a>(&self, id: &'a FileId) -> Option<Box<dyn std::fmt::Display + 'a>> {
        // Kinda unfortunate API limitation: we have to use a `Box<String>`
        // as `Box<str>` doesn't support casting to `dyn Display`. We have to allocate
        // because the lifetimes on this trait reference the file ID, not `self`.
        // Ref https://github.com/zesterer/ariadne/issues/10
        // Ariadne assumes the `id` is meaningful to users, but in apollo-rs it's
        // an incrementing integer, and file paths are stored separately.
        self.paths
            .get(id)
            .and_then(|path| path.to_str())
            .map(ToOwned::to_owned)
            .map(Box::new)
            .map(|bx| bx as Box<dyn std::fmt::Display + 'static>)
    }
}

// The default Debug impl is very verbose. The important part is that all
// files are in it, so just print those.
impl std::fmt::Debug for SourceCache {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_map()
            .entries({
                let mut paths: Vec<_> = self.paths.iter().collect();
                paths.sort_by(|a, b| a.0.cmp(b.0));
                paths.into_iter().map(|(id, path)| (id, path))
            })
            .finish()
    }
}

#[salsa::query_group(InputStorage)]
pub trait InputDatabase {
    /// Get input source of the corresponding file.
    #[salsa::input]
    fn input(&self, file_id: FileId) -> Source;

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
