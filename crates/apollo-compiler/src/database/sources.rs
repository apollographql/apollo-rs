use std::{
    path::{Path, PathBuf},
    sync::{atomic, Arc},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SourceType {
    Schema,
    Query,
    Document,
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

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
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

    /// Reset file ID back to 0, used to get consistent results in tests.
    #[allow(unused)]
    pub(crate) fn reset() {
        NEXT.store(0, atomic::Ordering::SeqCst);
    }
}
