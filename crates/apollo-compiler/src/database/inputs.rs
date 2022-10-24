#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub struct FileId(pub u32);

impl std::hash::Hash for FileId {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

#[salsa::query_group(InputStorage)]
pub trait InputDatabase {
    #[salsa::input]
    fn recursion_limit(&self) -> Option<usize>;
    #[salsa::input]
    fn input_document(&self) -> String;

    // @lrlna: Arc<String> ?
    #[salsa::input]
    fn input_schema(&self, file_id: FileId) -> String;

    #[salsa::input]
    fn input_query(&self, file_id: FileId) -> String;
}

// #[derive(Clone, Debug, PartialEq, Eq, Hash, Default)]
// pub struct Sources {
//     manifest: Vec<SourceInfo>,
// }
//
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
//
// #[derive(Clone, Debug, PartialEq, Eq, Hash)]
// pub struct SourceInfo {
//     pub(crate) file_id: FileId,
//     pub(crate) name: String,
// }
//
// impl SourceInfo {
//     pub fn new(name: String) -> Self {
//         Self {
//             file_id: FileId(Uuid::new_v4()),
//             name,
//         }
//     }
// }
//
// #[derive(Clone, Debug, PartialEq, Eq, Hash)]
// pub struct FileId(Uuid);
//
