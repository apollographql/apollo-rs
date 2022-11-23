use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Arc,
};

use uuid::Uuid;

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
    text: Arc<str>,
}

impl Source {
    /// Create a GraphQL schema source file.
    pub fn schema(text: impl Into<Arc<str>>) -> Self {
        Self {
            ty: SourceType::Schema,
            text: text.into(),
        }
    }

    /// Create a GraphQL query source file.
    pub fn query(text: impl Into<Arc<str>>) -> Self {
        Self {
            ty: SourceType::Query,
            text: text.into(),
        }
    }

    /// Create a GraphQL document source file.
    ///
    /// A Document can contain type definitions *and* queries.
    pub fn document(text: impl Into<Arc<str>>) -> Self {
        Self {
            ty: SourceType::Document,
            text: text.into(),
        }
    }

    pub fn source_type(&self) -> SourceType {
        self.ty
    }

    pub fn text(&self) -> Arc<str> {
        Arc::clone(&self.text)
    }
}

#[derive(Clone, Debug, Default)]
pub struct SourceManifest {
    pub(crate) manifest: HashMap<FileId, PathBuf>,
}

impl SourceManifest {
    pub fn add_source(&mut self, path: impl AsRef<Path>) -> FileId {
        let file_id = FileId(Uuid::new_v4());
        self.manifest.insert(file_id, path.as_ref().into());

        file_id
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct FileId(Uuid);
