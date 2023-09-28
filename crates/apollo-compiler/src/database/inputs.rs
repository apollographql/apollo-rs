use super::sources::{FileId, Source, SourceType};
use crate::hir::TypeSystem;
use ariadne::{Cache as AriadneCache, Source as AriadneSource};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Debug, thiserror::Error)]
#[error("Unknown file ID")]
struct UnknownFileError;

#[derive(Debug, Clone, PartialEq, Eq)]
struct ByteCharIndex {
    map: Vec<u32>,
}

impl ByteCharIndex {
    fn new(input: &str) -> Self {
        let mut map = vec![0; input.len() + 1];
        let mut char_index = 0;
        for (byte_index, _) in input.char_indices() {
            map[byte_index] = char_index;
            char_index += 1;
        }

        // Support 1 past the end of the string, for use in exclusive ranges.
        map[input.len()] = char_index;

        Self { map }
    }
}

/// Source text structure with a precomputed index from byte offsets to character offsets and to
/// line/column numbers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MappedSource {
    ariadne: AriadneSource,
    index: ByteCharIndex,
}

impl MappedSource {
    fn new(input: &str) -> Self {
        let ariadne = AriadneSource::from(input);
        let index = ByteCharIndex::new(input);

        Self { ariadne, index }
    }

    pub(crate) fn map_index(&self, byte_index: usize) -> usize {
        self.index.map[byte_index] as usize
    }
}

/// A Cache implementation for `ariadne` diagnostics.
///
/// Use [`InputDatabase::source_cache`] to construct one.
#[derive(Clone, PartialEq, Eq)]
pub struct SourceCache {
    sources: HashMap<FileId, Arc<MappedSource>>,
    paths: HashMap<FileId, PathBuf>,
}

impl SourceCache {
    pub(crate) fn get_source(&self, id: FileId) -> Option<&'_ MappedSource> {
        self.sources.get(&id).map(|arc| arc.as_ref())
    }

    pub fn get_line_column(&self, id: FileId, index: usize) -> Option<(usize, usize)> {
        let source = self.sources.get(&id)?;
        let char_index = source.map_index(index);
        let (_, line, column) = source.ariadne.get_offset_line(char_index)?;
        Some((line, column))
    }
}

impl AriadneCache<FileId> for &SourceCache {
    fn fetch(&mut self, id: &FileId) -> Result<&AriadneSource, Box<dyn std::fmt::Debug>> {
        let source = self.sources.get(id);
        source
            .map(|arc| &arc.ariadne)
            .ok_or_else(|| Box::new(UnknownFileError) as Box<dyn std::fmt::Debug>)
    }
    fn display<'a>(&self, id: &'a FileId) -> Option<Box<dyn std::fmt::Display + 'a>> {
        // Kinda unfortunate API limitation: we have to use a `Box<String>`
        // as `Box<str>` doesn't support casting to `dyn Display`. We have to allocate
        // because the lifetimes on this trait reference the file ID, not `self`.
        // Ref https://github.com/zesterer/ariadne/issues/10
        // Ariadne assumes the `id` is meaningful to users, but in apollo-rs it's
        // an incrementing integer, and file paths are stored separately.
        let path = self.paths.get(id)?;
        let boxed = Box::new(path.to_str()?.to_string());
        Some(boxed)
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
                paths.into_iter().map(|(id, path)| (id.as_u64(), path))
            })
            .finish()
    }
}

#[salsa::query_group(InputStorage)]
pub trait InputDatabase {
    /// Get the currently set recursion limit.
    #[salsa::input]
    fn recursion_limit(&self) -> Option<usize>;

    /// Get the currently set token limit.
    #[salsa::input]
    fn token_limit(&self) -> Option<usize>;

    /// Get input source of the corresponding file.
    #[salsa::input]
    fn type_system_hir_input(&self) -> Option<Arc<TypeSystem>>;

    #[salsa::input]
    fn input(&self, file_id: FileId) -> Source;

    /// Get the GraphQL source text for a file.
    #[salsa::invoke(source_code)]
    fn source_code(&self, file_id: FileId) -> Arc<str>;

    /// Get the source type (document/schema/executable) for a file.
    #[salsa::invoke(source_type)]
    fn source_type(&self, file_id: FileId) -> SourceType;

    /// Get all file ids currently in the compiler.
    #[salsa::input]
    fn source_files(&self) -> Vec<FileId>;

    /// Find source file by file name.
    fn source_file(&self, path: PathBuf) -> Option<FileId>;

    /// Get the GraphQL source text for a file, split up into lines for
    /// printing diagnostics.
    #[salsa::invoke(source_with_lines)]
    fn source_with_lines(&self, file_id: FileId) -> Arc<MappedSource>;

    /// Get all GraphQL sources known to the compiler, split up into lines
    /// for printing diagnostics.
    #[salsa::invoke(source_cache)]
    fn source_cache(&self) -> Arc<SourceCache>;

    /// Get all type system definition (GraphQL schema) files.
    #[salsa::invoke(type_definition_files)]
    fn type_definition_files(&self) -> Vec<FileId>;

    /// Get all executable definition (GraphQL query) files.
    #[salsa::invoke(executable_definition_files)]
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

fn source_file(db: &dyn InputDatabase, path: PathBuf) -> Option<FileId> {
    db.source_files()
        .iter()
        .find(|id| db.input(**id).filename() == path)
        .copied()
}

fn source_type(db: &dyn InputDatabase, file_id: FileId) -> SourceType {
    db.input(file_id).source_type()
}

fn source_with_lines(db: &dyn InputDatabase, file_id: FileId) -> Arc<MappedSource> {
    let code = db.source_code(file_id);
    Arc::new(MappedSource::new(&code))
}

fn source_cache(db: &dyn InputDatabase) -> Arc<SourceCache> {
    let file_ids = db.source_files();
    let sources = file_ids
        .iter()
        .map(|&id| (id, db.source_with_lines(id)))
        .collect();
    let paths = file_ids
        .iter()
        .map(|&id| (id, db.input(id).filename().to_owned()))
        .collect();
    Arc::new(SourceCache { sources, paths })
}

fn type_definition_files(db: &dyn InputDatabase) -> Vec<FileId> {
    db.source_files()
        .into_iter()
        .filter(|source| {
            matches!(
                db.source_type(*source),
                SourceType::Schema | SourceType::Document | SourceType::BuiltIn
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
