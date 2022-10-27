pub mod db;
pub mod hir;

mod ast;
mod def;
mod document;
mod inputs;

pub use ast::{AstDatabase, AstStorage};
pub use db::RootDatabase;
pub use def::{HirDatabase, HirStorage};
pub use document::{DocumentDatabase, DocumentStorage};
pub use inputs::{InputDatabase, InputStorage};
