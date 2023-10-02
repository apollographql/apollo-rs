use super::sources::SourceType;
use super::InputDatabase;
use crate::ast;
use crate::schema::Name;
use crate::Arc;
use crate::FileId;
use crate::SourceFile;
use std::collections::HashMap;
use std::collections::HashSet;

/// Queries for parsing into the various in-memory representations of GraphQL documents
#[salsa::query_group(ReprStorage)]
pub trait ReprDatabase: InputDatabase {
    #[salsa::invoke(ast_parse_result)]
    #[doc(hidden)]
    fn _ast_parse_result(&self, file_id: FileId) -> Arc<ParseResult>;

    #[salsa::invoke(ast)]
    #[salsa::transparent]
    fn ast(&self, file_id: FileId) -> Arc<ast::Document>;

    #[salsa::invoke(recursion_reached)]
    #[salsa::transparent]
    fn recursion_reached(&self, file_id: FileId) -> usize;

    #[salsa::invoke(tokens_reached)]
    #[salsa::transparent]
    fn tokens_reached(&self, file_id: FileId) -> usize;

    #[salsa::invoke(schema)]
    fn schema(&self) -> Arc<crate::Schema>;

    #[salsa::invoke(executable_document)]
    fn executable_document(&self, file_id: FileId) -> Arc<crate::ExecutableDocument>;

    // TODO: another database trait? what to name it?

    /// Returns a map of interface names to names of types that implement that interface
    ///
    /// `Schema` only stores the inverse relationship
    /// (in [`ObjectType::implements_interfaces`] and [`InterfaceType::implements_interfaces`]),
    /// so iterating the implementers of an interface requires a linear scan
    /// of all types in the schema.
    /// If that is repeated for multiple interfaces,
    /// gathering them all at once amorticizes that cost.
    #[salsa::invoke(implementers_map)]
    fn implementers_map(&self) -> Arc<HashMap<Name, HashSet<Name>>>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseResult {
    document: Arc<ast::Document>,
    recursion_reached: usize,
    tokens_reached: usize,
}

fn ast_parse_result(db: &dyn ReprDatabase, file_id: FileId) -> Arc<ParseResult> {
    let input = db.source_code(file_id);
    let mut parser = apollo_parser::Parser::new(&input);
    if let Some(limit) = db.recursion_limit() {
        parser = parser.recursion_limit(limit);
    }
    if let Some(limit) = db.token_limit() {
        parser = parser.token_limit(limit);
    }
    let tree = parser.parse();
    let recursion_reached = tree.recursion_limit().high;
    let tokens_reached = tree.token_limit().high;
    let source_file = Arc::new(SourceFile {
        path: db.input(file_id).filename,
        source_text: String::clone(&db.source_code(file_id)),
        parse_errors: tree.errors().cloned().collect(),
    });
    let document = Arc::new(ast::Document::from_cst(
        tree.document(),
        file_id,
        source_file,
    ));
    Arc::new(ParseResult {
        document,
        recursion_reached,
        tokens_reached,
    })
}

fn ast(db: &dyn ReprDatabase, file_id: FileId) -> Arc<ast::Document> {
    db._ast_parse_result(file_id).document.clone()
}

fn recursion_reached(db: &dyn ReprDatabase, file_id: FileId) -> usize {
    db._ast_parse_result(file_id).recursion_reached
}

fn tokens_reached(db: &dyn ReprDatabase, file_id: FileId) -> usize {
    db._ast_parse_result(file_id).tokens_reached
}

fn schema(db: &dyn ReprDatabase) -> Arc<crate::Schema> {
    if let Some(schema) = db.schema_input() {
        return schema;
    }

    let mut builder = crate::Schema::builder();
    for file_id in db.type_definition_files() {
        let executable_definitions_are_errors = db.source_type(file_id) != SourceType::Document;
        let ast = db.ast(file_id);
        builder.add_ast_document(&ast, executable_definitions_are_errors);
    }
    Arc::new(builder.build())
}

fn executable_document(db: &dyn ReprDatabase, file_id: FileId) -> Arc<crate::ExecutableDocument> {
    let source_type = db.source_type(file_id);
    let type_system_definitions_are_errors = source_type != SourceType::Document;
    let mut executable = crate::executable::from_ast::document_from_ast(
        Some(&db.schema()),
        &db.ast(file_id),
        type_system_definitions_are_errors,
    );
    if source_type == SourceType::Document {
        if let Some((_, source_file)) = &mut executable.source {
            // The same parse errors will be in db.schema().sources,
            // so they would be redundant here.
            source_file.make_mut().parse_errors = Vec::new()
        }
    }
    Arc::new(executable)
}

fn implementers_map(db: &dyn ReprDatabase) -> Arc<HashMap<Name, HashSet<Name>>> {
    Arc::new(db.schema().implementers_map())
}
