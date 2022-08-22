use uuid::Uuid;

#[salsa::query_group(InputsStorage)]
pub trait Inputs {
    #[salsa::input]
    fn document_manifest(&self) -> Manifest;

    #[salsa::input]
    fn input(&self, name: String) -> String;
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Default)]
pub struct Manifest {
    manifest: Vec<SourceInfo>,
}

impl Manifest {
    pub fn new() -> Self {
        Self {
            manifest: Default::default(),
        }
    }

    pub fn with_sources(sources: Vec<String>) -> Self {
        Self {
            manifest: sources.into_iter().map(|s| SourceInfo::new(s)).collect(),
        }
    }

    pub(crate) fn manifest(&self, source: String) {
        self.manifest.push(SourceInfo::new(source))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct SourceInfo {
    pub(crate) file_id: FileId,
    pub(crate) name: String,
}

impl SourceInfo {
    pub fn new(name: String) -> Self {
        Self {
            file_id: FileId(Uuid::new_v4()),
            name,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct FileId(Uuid);
