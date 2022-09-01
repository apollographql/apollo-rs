use std::sync::Arc;

use apollo_parser::{ast, Parser as ApolloParser, SyntaxTree};

use crate::{diagnostics::SyntaxError, queries::inputs_db::Inputs, ApolloDiagnostic};

#[salsa::query_group(ParserStorage)]
pub trait DocumentParser: Inputs {
    fn ast(&self) -> SyntaxTree;

    // root node
    fn document(&self) -> Arc<ast::Document>;

    fn syntax_errors(&self) -> Vec<ApolloDiagnostic>;
}

fn ast(db: &dyn DocumentParser) -> SyntaxTree {
    let input = db.input();

    let parser = ApolloParser::new(&input);
    parser.parse()
}

fn document(db: &dyn DocumentParser) -> Arc<ast::Document> {
    Arc::new(db.ast().document())
}

fn syntax_errors(db: &dyn DocumentParser) -> Vec<ApolloDiagnostic> {
    db.ast()
        .errors()
        .into_iter()
        .map(|err| {
            ApolloDiagnostic::SyntaxError(SyntaxError {
                src: db.input(),
                span: (err.index(), err.data().len()).into(), // (offset, length of error token)
                message: err.message().into(),
            })
        })
        .collect()
}
