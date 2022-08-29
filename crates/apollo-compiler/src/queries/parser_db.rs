use std::sync::Arc;

use apollo_parser::{ast, Parser as ApolloParser, SyntaxTree};

use crate::queries::inputs_db::Inputs;

#[salsa::query_group(ParserStorage)]
pub trait DocumentParser: Inputs {
    fn ast(&self) -> SyntaxTree;

    // root node
    fn document(&self) -> Arc<ast::Document>;
}

fn document(db: &dyn DocumentParser) -> Arc<ast::Document> {
    Arc::new(db.ast().document())
}

fn ast(db: &dyn DocumentParser) -> SyntaxTree {
    let input = db.input();

    let parser = ApolloParser::new(&input);
    parser.parse()
}
