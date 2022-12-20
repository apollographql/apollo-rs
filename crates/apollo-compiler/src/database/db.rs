// All .expect() calls are used for parts of the GraphQL grammar that are
// non-optional and will have an error produced in the parser if they are missing.

use crate::{
    database::{AstStorage, HirStorage, InputStorage},
    validation::ValidationStorage,
    HirDatabase,
};

pub trait Upcast<T: ?Sized> {
    fn upcast(&self) -> &T;
}
#[salsa::database(InputStorage, AstStorage, HirStorage, ValidationStorage)]
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

impl Upcast<dyn HirDatabase> for RootDatabase {
    fn upcast(&self) -> &(dyn HirDatabase + 'static) {
        self
    }
}
