use crate::ast;
use crate::schema::Implementers;
use crate::schema::Name;
use std::collections::HashMap;
use std::sync::Arc;

/// Queries for parsing into the various in-memory representations of GraphQL documents
#[salsa::query_group(ReprStorage)]
pub(crate) trait ReprDatabase {
    #[salsa::input]
    fn schema(&self) -> Arc<crate::Schema>;

    #[salsa::input]
    fn executable_ast(&self) -> Arc<ast::Document>;

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

fn implementers_map(db: &dyn ReprDatabase) -> Arc<HashMap<Name, Implementers>> {
    Arc::new(db.schema().implementers_map())
}
