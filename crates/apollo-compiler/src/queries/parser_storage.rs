use apollo_parser::{Parser as ApolloParser, SyntaxTree};
use std::sync::Arc;

use crate::queries::inputs_storage::Inputs;

#[salsa::query_group(ParserStorage)]
pub trait DocumentParser: Inputs {
    fn ast(&self, name: String) -> Arc<SyntaxTree>;
}

fn ast(db: &dyn DocumentParser, name: String) -> Arc<SyntaxTree> {
    let input = db.input(name);

    let parser = ApolloParser::new(&input);
    Arc::new(parser.parse())
}
