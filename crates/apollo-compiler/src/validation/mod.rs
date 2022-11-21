mod validation_db;

mod directive;
mod enum_;
mod input_object;
mod interface;
mod object;
mod operation;
mod scalar;
mod schema;
mod union_;
mod unused_variable;

pub use validation_db::{ValidationDatabase, ValidationStorage};

use crate::hir::HirNodeLocation;

#[derive(Debug, Eq)]
struct ValidationSet {
    name: String,
    loc: HirNodeLocation,
}

impl std::hash::Hash for ValidationSet {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

impl PartialEq for ValidationSet {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}
