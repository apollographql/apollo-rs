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

pub use validation_db::{ValidationDatabase, ValidationStorage};

use crate::database::HirDatabase;
use crate::hir::HirNodeLocation;
use apollo_parser::cst::CstNode;

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

/// Find the closest CST node of the requested type that contains the whole range indicated by `location`.
fn lookup_cst_node<T: CstNode>(db: &dyn HirDatabase, location: HirNodeLocation) -> Option<T> {
    let document = db.cst(location.file_id).document();
    let root = document.syntax();
    let element = root.covering_element(location.text_range);
    element.ancestors().find_map(T::cast)
}

/// Create a custom text range based on the concrete syntax tree.
fn lookup_cst_location<T: CstNode>(
    db: &dyn HirDatabase,
    reference_location: HirNodeLocation,
    build_range: impl Fn(T) -> Option<apollo_parser::TextRange>,
) -> Option<HirNodeLocation> {
    let node = lookup_cst_node::<T>(db, reference_location)?;
    build_range(node).map(|text_range| HirNodeLocation {
        file_id: reference_location.file_id,
        text_range,
    })
}
