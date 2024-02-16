// All .expect() calls are used for parts of the GraphQL grammar that are
// non-optional and will have an error produced in the parser if they are missing.

use crate::database::ReprStorage;

#[salsa::database(ReprStorage)]
#[derive(Default)]
pub struct RootDatabase {
    pub storage: salsa::Storage<RootDatabase>,
}

impl salsa::Database for RootDatabase {}

// NOTE @lrlna: uncomment when we are fully thread-safe.

impl salsa::ParallelDatabase for RootDatabase {
    fn snapshot(&self) -> salsa::Snapshot<RootDatabase> {
        salsa::Snapshot::new(RootDatabase {
            storage: self.storage.snapshot(),
        })
    }
}
