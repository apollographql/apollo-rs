mod validation_db;

mod argument;
mod directive;
mod enum_;
mod extension;
mod field;
mod fragment;
mod input_object;
mod interface;
mod object;
mod operation;
mod scalar;
mod schema;
mod selection;
mod union_;
mod variable;

pub use validation_db::{ValidationDatabase, ValidationStorage};

use crate::hir::HirNodeLocation;

#[derive(Debug, Eq)]
struct ValidationSet {
    name: String,
    loc: Option<HirNodeLocation>,
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
