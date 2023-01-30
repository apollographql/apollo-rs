pub mod db;
pub mod hir;

mod ast;
mod document;
mod hir_db;
mod inputs;
mod sources;

pub use ast::{AstDatabase, AstStorage};
pub use db::RootDatabase;
pub use hir_db::{HirDatabase, HirStorage};
pub use inputs::{InputDatabase, InputStorage, SourceCache};
pub use sources::{FileId, Source};
