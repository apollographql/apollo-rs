use apollo_parser::{Parser as ApolloParser, SyntaxTree};
use rowan::GreenNode;

use crate::{database::inputs::InputDatabase, diagnostics::SyntaxError, ApolloDiagnostic, FileId};

#[salsa::query_group(AstStorage)]
pub trait AstDatabase: InputDatabase {
    /// Get an AST for a particular file. Returns a `rowan` SyntaxTree.  The
    /// SyntaxTree can be safely shared between threads as it's `Send` and
    /// `Sync`.
    fn ast(&self, file_id: FileId) -> SyntaxTree;

    /// Get a file's GraphQL Document. Returns a `rowan` Green Node. This is the
    /// top level document node that can be used when going between an
    /// SyntaxNodePtr to an actual SyntaxNode.
    fn document(&self, file_id: FileId) -> GreenNode;

    /// Get syntax errors found in the compiler's manifest.
    fn syntax_errors(&self) -> Vec<ApolloDiagnostic>;
}

fn ast(db: &dyn AstDatabase, file_id: FileId) -> SyntaxTree {
    let input = db.source_code(file_id);

    let parser = ApolloParser::new(&input);
    let parser = if let Some(limit) = db.recursion_limit() {
        parser.recursion_limit(limit)
    } else {
        parser
    };
    parser.parse()
}

fn document(db: &dyn AstDatabase, file_id: FileId) -> GreenNode {
    db.ast(file_id).green()
}

fn syntax_errors(db: &dyn AstDatabase) -> Vec<ApolloDiagnostic> {
    db.source_files()
        .into_iter()
        .flat_map(|file_id| {
            db.ast(file_id)
                .errors()
                .into_iter()
                .map(|err| {
                    ApolloDiagnostic::SyntaxError(SyntaxError {
                        src: db.source_code(file_id),
                        span: (err.index(), err.data().len()).into(), // (offset, length of error token)
                        message: err.message().into(),
                    })
                })
                .collect::<Vec<ApolloDiagnostic>>()
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
        let mut compiler = ApolloCompiler::with_recursion_limit(1);
        let doc_id = compiler.create_document(schema, "schema.graphql");

        let ast = compiler.db.ast(doc_id);

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
        let mut compiler = ApolloCompiler::with_recursion_limit(7);
        let doc_id = compiler.create_document(schema, "schema.graphql");

        let ast = compiler.db.ast(doc_id);

        assert_eq!(ast.recursion_limit().high, 4);
        assert_eq!(ast.errors().len(), 0);
        assert_eq!(ast.document().definitions().into_iter().count(), 1);
    }
}
