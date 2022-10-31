pub mod db;
pub mod hir;

mod ast;
mod document;
mod hir_db;
mod inputs;

pub use ast::{AstDatabase, AstStorage};
pub use db::RootDatabase;
pub use document::{DocumentDatabase, DocumentStorage};
pub use hir_db::{HirDatabase, HirStorage};
pub use inputs::{InputDatabase, InputStorage};
