use apollo_parser::{Parser as ApolloParser, SyntaxTree};
use rowan::GreenNode;

use crate::database::inputs::InputDatabase;
use crate::diagnostics::{ApolloDiagnostic, DiagnosticData, Label};
use crate::FileId;

#[salsa::query_group(CstStorage)]
pub trait CstDatabase: InputDatabase {
    /// Get an AST for a particular file. Returns a `rowan` SyntaxTree.  The
    /// SyntaxTree can be safely shared between threads as it's `Send` and
    /// `Sync`.
    #[salsa::invoke(cst)]
    fn cst(&self, file_id: FileId) -> SyntaxTree;

    /// Get a file's GraphQL Document. Returns a `rowan` Green Node. This is the
    /// top level document node that can be used when going between an
    /// SyntaxNodePtr to an actual SyntaxNode.
    #[salsa::invoke(document)]
    fn document(&self, file_id: FileId) -> GreenNode;

    /// Get syntax errors found in the compiler's manifest.
    #[salsa::invoke(syntax_errors)]
    fn syntax_errors(&self) -> Vec<ApolloDiagnostic>;
}

fn cst(db: &dyn CstDatabase, file_id: FileId) -> SyntaxTree {
    // Do not use `db.source_code(file_id)` here
    // as that would also include sources of for pre-computed input,
    // which we don’t want to re-parse.
    let input = db.input(file_id);

    let mut parser = ApolloParser::new(input.text());
    if let Some(limit) = db.recursion_limit() {
        parser = parser.recursion_limit(limit)
    };
    if let Some(limit) = db.token_limit() {
        parser = parser.token_limit(limit)
    };

    parser.parse()
}

fn document(db: &dyn CstDatabase, file_id: FileId) -> GreenNode {
    db.cst(file_id).green()
}

fn syntax_errors(db: &dyn CstDatabase) -> Vec<ApolloDiagnostic> {
    db.source_files()
        .into_iter()
        .flat_map(|file_id| {
            db.cst(file_id)
                .errors()
                .map(|err| {
                    if err.is_limit() {
                        ApolloDiagnostic::new(
                            db,
                            (file_id, err.index(), err.data().len()).into(),
                            DiagnosticData::LimitExceeded {
                                message: err.message().into(),
                            },
                        )
                        .label(Label::new(
                            (file_id, err.index(), err.data().len()),
                            err.message(),
                        ))
                    } else {
                        ApolloDiagnostic::new(
                            db,
                            (file_id, err.index(), err.data().len()).into(),
                            DiagnosticData::SyntaxError {
                                message: err.message().into(),
                            },
                        )
                        .label(Label::new(
                            (file_id, err.index(), err.data().len()),
                            err.message(),
                        ))
                    }
                })
                .collect::<Vec<ApolloDiagnostic>>()
        })
        .collect()
}
