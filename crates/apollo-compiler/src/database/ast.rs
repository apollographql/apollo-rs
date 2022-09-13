use apollo_parser::{Parser as ApolloParser, SyntaxTree};
use rowan::GreenNode;

use crate::{database::inputs::Inputs, diagnostics::SyntaxError, ApolloDiagnostic};

#[salsa::query_group(ParserStorage)]
pub trait DocumentParser: Inputs {
    fn ast(&self) -> SyntaxTree;

    // root node
    fn document(&self) -> GreenNode;

    fn syntax_errors(&self) -> Vec<ApolloDiagnostic>;
}

fn ast(db: &dyn DocumentParser) -> SyntaxTree {
    let input = db.input();

    let parser = ApolloParser::new(&input);
    parser.parse()
}

fn document(db: &dyn DocumentParser) -> GreenNode {
    db.ast().green()
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
