use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SourceType {
    Schema,
    Query,
    Document,
}

/// Represents a GraphQL source file.
#[derive(Clone, Debug, Hash)]
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
pub struct FileId(pub u32);
