// All .expect() calls are used for parts of the GraphQL grammar that are
// non-optional and will have an error produced in the parser if they are missing.

use crate::queries::{
    def_db::DefinitionsStorage, document_db::DocumentStorage, inputs_db::InputsStorage,
    parser_db::ParserStorage,
};

#[salsa::database(DocumentStorage, InputsStorage, ParserStorage, DefinitionsStorage)]
#[derive(Default)]
pub struct RootDatabase {
    pub storage: salsa::Storage<RootDatabase>,
}

impl salsa::Database for RootDatabase {}

// NOTE @lrlna: uncomment when we are fully thread-safe.

// impl salsa::ParallelDatabase for RootDatabase {
//     fn snapshot(&self) -> salsa::Snapshot<DocumentDatabase> {
//         salsa::Snapshot::new(DocumentDatabase {
//             storage: self.storage.snapshot(),
//         })
//     }
// }
