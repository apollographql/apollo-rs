pub mod db;
pub mod hir;

mod ast;
mod def;
mod document;
mod inputs;

pub use ast::AstDatabase;
pub use db::RootDatabase;
pub use def::HirDatabase;
pub use document::DocumentDatabase;
pub use inputs::InputDatabase;
