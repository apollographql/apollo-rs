use apollo_parser::{ast, Parser as ApolloParser, SyntaxTree};
use std::sync::Arc;

use crate::queries::document_storage::Inputs;

#[salsa::query_group(ParserStorage)]
pub trait DocumentParser: Inputs {
    fn ast(&self, name: String) -> Arc<SyntaxTree>;

    fn syntax_errors(&self) -> Vec<Error>;
}

fn ast(db: &dyn DocumentParser, name: String) -> Arc<SyntaxTree> {
    let input = db.document(name);

    let parser = ApolloParser::new(&input);
    Arc::new(parser.parse())
}

fn syntax_errors(db: &dyn DocumentParser) -> Arc<Vec<Error>> {
    db.ast().errors()
}
