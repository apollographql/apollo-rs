use super::InputDatabase;
use crate::ast;
use crate::diagnostics::DiagnosticData;
use crate::diagnostics::Label;
use crate::ApolloDiagnostic;
use crate::Arc;
use crate::FileId;

/// Queries for parsing into the various in-memory representations of GraphQL documents
#[salsa::query_group(ReprStorage)]
pub trait ReprDatabase: InputDatabase {
    #[salsa::invoke(ast_parse_result)]
    #[doc(hidden)]
    fn _ast_parse_result(&self, file_id: FileId) -> Arc<ParseResult>;

    #[salsa::invoke(ast)]
    #[salsa::transparent]
    fn ast(&self, file_id: FileId) -> Arc<ast::Document>;

    #[salsa::invoke(syntax_errors)]
    #[salsa::transparent]
    fn syntax_errors(&self, file_id: FileId) -> Arc<Vec<ApolloDiagnostic>>;

    #[salsa::invoke(recursion_reached)]
    #[salsa::transparent]
    fn recursion_reached(&self, file_id: FileId) -> usize;

    #[salsa::invoke(tokens_reached)]
    #[salsa::transparent]
    fn tokens_reached(&self, file_id: FileId) -> usize;

    #[salsa::invoke(schema)]
    fn schema(&self) -> Arc<crate::Schema>;

    #[salsa::invoke(executable_document)]
    fn executable_document(
        &self,
        file_id: FileId,
    ) -> Result<Arc<crate::ExecutableDocument>, crate::executable::TypeError>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseResult {
    document: Arc<ast::Document>,
    syntax_errors: Arc<Vec<ApolloDiagnostic>>,
    recursion_reached: usize,
    tokens_reached: usize,
}

fn ast_parse_result(db: &dyn ReprDatabase, file_id: FileId) -> Arc<ParseResult> {
    let mut parser = ast::Document::parser();
    if let Some(limit) = db.recursion_limit() {
        parser.recursion_limit(limit);
    }
    if let Some(limit) = db.token_limit() {
        parser.token_limit(limit);
    }
    let result = parser.parse(&db.source_code(file_id));
    Arc::new(ParseResult {
        document: result.document,
        recursion_reached: result.recursion_reached,
        tokens_reached: result.tokens_reached,
        syntax_errors: Arc::new(
            result
                .syntax_errors
                .into_iter()
                .map(|err| {
                    let err = err.0;
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
                .collect(),
        ),
    })
}

fn ast(db: &dyn ReprDatabase, file_id: FileId) -> Arc<ast::Document> {
    db._ast_parse_result(file_id).document.clone()
}

fn syntax_errors(db: &dyn ReprDatabase, file_id: FileId) -> Arc<Vec<ApolloDiagnostic>> {
    db._ast_parse_result(file_id).syntax_errors.clone()
}

fn recursion_reached(db: &dyn ReprDatabase, file_id: FileId) -> usize {
    db._ast_parse_result(file_id).recursion_reached
}

fn tokens_reached(db: &dyn ReprDatabase, file_id: FileId) -> usize {
    db._ast_parse_result(file_id).tokens_reached
}

fn schema(db: &dyn ReprDatabase) -> Arc<crate::Schema> {
    let mut builder = crate::Schema::builder();
    for file_id in db.type_definition_files() {
        builder.add_document(&db.ast(file_id))
    }
    let (schema, _orphan_extensions) = builder.build();
    Arc::new(schema)
}

fn executable_document(
    db: &dyn ReprDatabase,
    file_id: FileId,
) -> Result<Arc<crate::ExecutableDocument>, crate::executable::TypeError> {
    crate::ExecutableDocument::from_ast(&db.schema(), &db.ast(file_id)).map(Arc::new)
}
