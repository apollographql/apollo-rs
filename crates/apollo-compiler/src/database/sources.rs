use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use uuid::Uuid;

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
