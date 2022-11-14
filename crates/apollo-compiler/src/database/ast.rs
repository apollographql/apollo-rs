use apollo_parser::{Parser as ApolloParser, SyntaxTree};
use rowan::GreenNode;

use crate::{database::inputs::InputDatabase, diagnostics::SyntaxError, ApolloDiagnostic};

#[salsa::query_group(AstStorage)]
pub trait AstDatabase: InputDatabase {
    fn ast(&self) -> SyntaxTree;

    // root node
    fn document(&self) -> GreenNode;

    fn syntax_errors(&self) -> Vec<ApolloDiagnostic>;
}

fn ast(db: &dyn AstDatabase) -> SyntaxTree {
    let input = db.input();

    let parser = if let Some(limit) = db.recursion_limit() {
        ApolloParser::with_recursion_limit(&input, limit)
    } else {
        ApolloParser::new(&input)
    };
    parser.parse()
}

fn document(db: &dyn AstDatabase) -> GreenNode {
    db.ast().green()
}

fn syntax_errors(db: &dyn AstDatabase) -> Vec<ApolloDiagnostic> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ApolloCompiler;

    #[test]
    fn it_errors_when_selection_set_recursion_limit_exceeded() {
        let schema = r#"
        query {
          Q1 {
            url
          }
        }
        "#;
        let compiler = ApolloCompiler::with_recursion_limit(schema, 1);

        let ast = compiler.db.ast();

        assert_eq!(ast.recursion_limit().high, 2);
        assert_eq!(ast.errors().len(), 1);
        assert_eq!(ast.document().definitions().into_iter().count(), 2);
    }

    #[test]
    fn it_passes_when_selection_set_recursion_limit_is_not_exceeded() {
        let schema = r#"
        query {
          Q1 {
            url
          }
        }
        "#;
        let compiler = ApolloCompiler::with_recursion_limit(schema, 7);

        let ast = compiler.db.ast();

        assert_eq!(ast.recursion_limit().high, 4);
        assert_eq!(ast.errors().len(), 0);
        assert_eq!(ast.document().definitions().into_iter().count(), 1);
    }
}
