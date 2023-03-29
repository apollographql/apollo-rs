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

/// Track used names in a recursive function.
///
/// Pass the result of `stack.push(name)` to recursive calls. Use `stack.contains(name)` to check
/// if the name was used somewhere up the call stack.
struct RecursionStack<'a>(&'a mut Vec<String>);
impl RecursionStack<'_> {
    fn push(&mut self, name: String) -> RecursionStack<'_> {
        self.0.push(name);
        RecursionStack(self.0)
    }
    fn contains(&self, name: &str) -> bool {
        self.0.iter().any(|seen| seen == name)
    }
    fn first(&self) -> Option<&str> {
        self.0.get(0).map(|s| s.as_str())
    }
}
impl Drop for RecursionStack<'_> {
    fn drop(&mut self) {
        self.0.pop();
    }
}
