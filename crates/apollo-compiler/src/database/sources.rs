use std::{collections::HashSet, hash::BuildHasherDefault, path::PathBuf};

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct FileId(pub u32);

// #[derive(Clone, Debug, PartialEq, Eq, Hash, Default)]
// pub struct Sources {
//     interner: SourceInterner,
//     manifest: Vec<SourceInfo>,
// }

// impl Sources {
//     pub fn new(source: String) -> Self {
//         Self {
//             manifest: vec![SourceInfo::new(source)],
//         }
//     }
//
//     pub fn with_sources(sources: Vec<String>) -> Self {
//         Self {
//             manifest: sources.into_iter().map(|s| SourceInfo::new(s)).collect(),
//         }
//     }
//
//     pub(crate) fn manifest(&self, source: String) {
//         self.manifest.push(SourceInfo::new(source))
//     }
// }
// #[derive(Clone, Debug, PartialEq, Eq, Hash)]
// pub struct SourceInfo {
//     pub(crate) file_id: FileId,
//     // @lrlna probably `path`?
//     pub(crate) name: String,
// }

// impl SourceInfo {
//     pub fn new(name: String) -> Self {
//         Self {
//             file_id: FileId(Uuid::new_v4()),
//             name,
//         }
//     }
// }
//
// pub(crate) struct SourceInterner {
//     map: HashSet<SourcePath, BuildHasherDefault<FxHasher>>,
// }

pub struct SourcePath(PathBuf);
