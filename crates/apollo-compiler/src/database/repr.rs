use super::sources::SourceType;
use super::InputDatabase;
use crate::ast;
use crate::schema::Implementers;
use crate::schema::Name;
use crate::validation::DiagnosticList;
use crate::validation::FileId;
use std::collections::HashMap;
use std::sync::Arc;

/// Queries for parsing into the various in-memory representations of GraphQL documents
#[salsa::query_group(ReprStorage)]
pub(crate) trait ReprDatabase: InputDatabase {
    #[salsa::invoke(ast)]
    #[salsa::transparent]
    fn ast(&self, file_id: FileId) -> Arc<ast::Document>;

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
    fn implementers_map(&self) -> Arc<HashMap<Name, Implementers>>;
}

fn ast(db: &dyn ReprDatabase, file_id: FileId) -> Arc<ast::Document> {
    db.input(file_id).ast.clone().unwrap()
}

fn executable_document(db: &dyn ReprDatabase, file_id: FileId) -> Arc<crate::ExecutableDocument> {
    let source_type = db.source_type(file_id);
    let type_system_definitions_are_errors = source_type != SourceType::Document;
    let mut errors = DiagnosticList::new(Default::default());
    let executable = crate::executable::from_ast::document_from_ast(
        Some(&db.schema()),
        &db.ast(file_id),
        &mut errors,
        type_system_definitions_are_errors,
    );
    Arc::new(executable)
}

fn implementers_map(db: &dyn ReprDatabase) -> Arc<HashMap<Name, Implementers>> {
    Arc::new(db.schema().implementers_map())
}
