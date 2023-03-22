use std::{
    path::{Path, PathBuf},
    sync::{atomic, Arc},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SourceType {
    Schema,
    Query,
    Document,
    BuiltIn,
}

impl SourceType {
    /// Returns `true` if the source type is [`BuiltIn`].
    ///
    /// [`BuiltIn`]: SourceType::BuiltIn
    #[must_use]
    pub fn is_built_in(&self) -> bool {
        matches!(self, Self::BuiltIn)
    }

    /// Returns `true` if the source type is [`Document`].
    ///
    /// [`Document`]: SourceType::Document
    #[must_use]
    pub fn is_document(&self) -> bool {
        matches!(self, Self::Document)
    }

    /// Returns `true` if the source type is [`Query`].
    ///
    /// [`Query`]: SourceType::Query
    #[must_use]
    pub fn is_query(&self) -> bool {
        matches!(self, Self::Query)
    }

    /// Returns `true` if the source type is [`Schema`].
    ///
    /// [`Schema`]: SourceType::Schema
    #[must_use]
    pub fn is_schema(&self) -> bool {
        matches!(self, Self::Schema)
    }
}

/// Represents a GraphQL source file.
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Source {
    ty: SourceType,
    filename: PathBuf,
    text: Arc<str>,
}

impl Source {
    /// Create a GraphQL schema source file.
    pub fn schema(filename: PathBuf, text: impl Into<Arc<str>>) -> Self {
        Self {
            ty: SourceType::Schema,
            filename,
            text: text.into(),
        }
    }

    /// Create a GraphQL executable source file.
    pub fn executable(filename: PathBuf, text: impl Into<Arc<str>>) -> Self {
        Self {
            ty: SourceType::Query,
            filename,
            text: text.into(),
        }
    }

    /// Create a GraphQL document source file.
    ///
    /// A Document can contain type definitions *and* executable definitions. You can also use it
    /// when you don't know the actual source type.
    pub fn document(filename: PathBuf, text: impl Into<Arc<str>>) -> Self {
        Self {
            ty: SourceType::Document,
            filename,
            text: text.into(),
        }
    }
    /// Create a GraphQL type system file with built in types.
    pub(crate) fn built_in(filename: PathBuf, text: impl Into<Arc<str>>) -> Self {
        Self {
            ty: SourceType::BuiltIn,
            filename,
            text: text.into(),
        }
    }

    pub fn filename(&self) -> &Path {
        &self.filename
    }

    pub fn source_type(&self) -> SourceType {
        self.ty
    }

    pub fn text(&self) -> Arc<str> {
        Arc::clone(&self.text)
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct FileId {
    id: u64,
}

/// The next file ID to use. This is global so file IDs do not conflict between different compiler
/// instances.
static NEXT: atomic::AtomicU64 = atomic::AtomicU64::new(0);

impl FileId {
    // Returning a different value every time does not sound like good `impl Default`
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            id: NEXT.fetch_add(1, atomic::Ordering::Relaxed),
        }
    }

    // Exposed for tests, but relying on the test order is probably not a good idea…
    pub(crate) fn as_u64(self) -> u64 {
        self.id
    }

    /// Reset file ID back to 0, used to get consistent results in tests.
    #[allow(unused)]
    pub(crate) fn reset() {
        NEXT.store(0, atomic::Ordering::SeqCst);
    }
}
