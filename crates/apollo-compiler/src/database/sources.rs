use crate::Arc;
use std::{
    num::NonZeroI64,
    path::{Path, PathBuf},
    sync::atomic,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SourceType {
    Schema,
    Executable,
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

    /// Returns `true` if the source type is [`Executable`].
    ///
    /// [`Executable`]: SourceType::Executable
    #[must_use]
    pub fn is_executable(&self) -> bool {
        matches!(self, Self::Executable)
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
    pub(crate) filename: PathBuf,
    text: Arc<String>,
}

impl Source {
    /// Create a GraphQL schema source file.
    pub fn schema(filename: PathBuf, text: impl Into<String>) -> Self {
        Self {
            ty: SourceType::Schema,
            filename,
            text: Arc::new(text.into()),
        }
    }

    /// Create a GraphQL executable source file.
    pub fn executable(filename: PathBuf, text: impl Into<String>) -> Self {
        Self {
            ty: SourceType::Executable,
            filename,
            text: Arc::new(text.into()),
        }
    }

    /// Create a GraphQL document source file.
    ///
    /// A Document can contain type definitions *and* executable definitions. You can also use it
    /// when you don't know the actual source type.
    pub fn document(filename: PathBuf, text: impl Into<String>) -> Self {
        Self {
            ty: SourceType::Document,
            filename,
            text: Arc::new(text.into()),
        }
    }

    /// Create a GraphQL type system file with built in types.
    // TODO(goto-bus-stop,SimonSapin): remove
    #[allow(unused)]
    #[deprecated = "New AST does not have this"]
    pub(crate) fn built_in(filename: PathBuf, text: impl Into<String>) -> Self {
        Self {
            ty: SourceType::BuiltIn,
            filename,
            text: Arc::new(text.into()),
        }
    }

    pub fn filename(&self) -> &Path {
        &self.filename
    }

    pub fn source_type(&self) -> SourceType {
        self.ty
    }

    pub fn text(&self) -> &Arc<String> {
        &self.text
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct FileId {
    id: NonZeroI64,
}

/// The next file ID to use. This is global so file IDs do not conflict between different compiler
/// instances.
static NEXT: atomic::AtomicI64 = atomic::AtomicI64::new(1);

impl FileId {
    /// The ID of the file implicitly added to type systems, for built-in scalars and introspection types
    pub const BUILT_IN: Self = Self::const_new(-1);

    // Returning a different value every time does not sound like good `impl Default`
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let id = NEXT.fetch_add(1, atomic::Ordering::Relaxed);
        Self {
            id: NonZeroI64::new(id).unwrap(),
        }
    }

    // Exposed for tests, but relying on the test order is probably not a good idea…
    pub(crate) fn to_i64(self) -> i64 {
        self.id.get()
    }

    /// Reset file ID back to 1, used to get consistent results in tests.
    #[cfg(test)]
    pub(crate) fn reset() {
        NEXT.store(1, atomic::Ordering::SeqCst);
    }

    const fn const_new(id: i64) -> Self {
        // TODO: use unwrap() when const-stable https://github.com/rust-lang/rust/issues/67441
        if let Some(id) = NonZeroI64::new(id) {
            Self { id }
        } else {
            panic!()
        }
    }
}
