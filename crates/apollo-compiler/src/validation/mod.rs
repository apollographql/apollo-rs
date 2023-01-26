mod validation_db;

mod argument;
mod directive;
mod enum_;
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

use crate::{hir::HirNodeLocation, FileId};
use apollo_parser::ast::AstNode;

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

/// Finds top-level AST nodes of a given type in type definition files.
///
/// This ignores pre-computed inputs.
pub(crate) fn ast_type_definitions<'db, AstType>(
    db: &'db dyn ValidationDatabase,
) -> impl Iterator<Item = (FileId, AstType)> + 'db
where
    AstType: 'db + AstNode,
{
    db.type_definition_files()
        .into_iter()
        .flat_map(move |file_id| {
            db.ast(file_id)
                .document()
                .syntax()
                .children()
                .filter_map(AstNode::cast)
                .map(move |def| (file_id, def))
        })
}
