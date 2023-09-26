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
mod value;
mod variable;

use crate::NodeStr;
pub use validation_db::{ValidationDatabase, ValidationStorage};

/// Track used names in a recursive function.
struct RecursionStack {
    seen: Vec<NodeStr>,
}

impl RecursionStack {
    fn new() -> Self {
        Self {
            seen: Default::default(),
        }
    }

    fn with_root(root: NodeStr) -> Self {
        Self { seen: vec![root] }
    }

    /// Return the actual API for tracking recursive uses.
    pub fn guard(&mut self) -> RecursionGuard<'_> {
        RecursionGuard(&mut self.seen)
    }
}

/// Track used names in a recursive function.
///
/// Pass the result of `guard.push(name)` to recursive calls. Use `guard.contains(name)` to check
/// if the name was used somewhere up the call stack. When a guard is dropped, its name is removed
/// from the list.
struct RecursionGuard<'a>(&'a mut Vec<NodeStr>);
impl RecursionGuard<'_> {
    /// Mark that we saw a name.
    fn push(&mut self, name: &NodeStr) -> RecursionGuard<'_> {
        self.0.push(name.clone());
        RecursionGuard(self.0)
    }
    /// Check if we saw a name somewhere up the call stack.
    fn contains(&self, name: &str) -> bool {
        self.0.iter().any(|seen| seen == name)
    }
    /// Return the name where we started.
    fn first(&self) -> Option<&str> {
        self.0.get(0).map(|s| s.as_str())
    }
}

impl Drop for RecursionGuard<'_> {
    fn drop(&mut self) {
        // This may already be empty if it's the original `stack.guard()` result, but that's fine
        self.0.pop();
    }
}
