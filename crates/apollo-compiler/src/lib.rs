#![doc = include_str!("../README.md")]

#[macro_use]
mod macros;
pub mod ast;
mod database;
pub mod diagnostic;
pub mod executable;
pub mod execution;
mod node;
mod node_str;
mod parser;
pub mod schema;
pub mod validation;

use crate::database::{InputDatabase, ReprDatabase, RootDatabase, Source};
use crate::validation::ValidationDatabase;

pub use self::executable::ExecutableDocument;
pub use self::node::Node;
pub use self::node_str::NodeStr;
pub use self::parser::{parse_mixed_validate, Parser, SourceFile, SourceMap};
pub use self::schema::Schema;

pub(crate) struct ApolloCompiler {
    pub db: RootDatabase,
}

/// Apollo compiler creates a context around your GraphQL. It creates references
/// between various GraphQL types in scope.
#[allow(clippy::new_without_default)]
impl ApolloCompiler {
    /// Create a new instance of Apollo Compiler.
    pub fn new() -> Self {
        let mut db = RootDatabase::default();
        // TODO(@goto-bus-stop) can we make salsa fill in these defaults for us…?
        db.set_source_files(vec![]);
        db.set_schema_input(None);

        Self { db }
    }
}
