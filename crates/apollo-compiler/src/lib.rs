#![doc = include_str!("../README.md")]

#[macro_use]
mod macros;
pub mod ast;
mod database;
mod diagnostics;
pub mod executable;
mod node;
mod node_str;
mod parser;
pub mod schema;
mod validation;

use crate::database::{InputDatabase, ReprDatabase, RootDatabase, Source};
use crate::diagnostics::ApolloDiagnostic;
use crate::validation::ValidationDatabase;

pub use self::database::FileId;
pub use self::executable::ExecutableDocument;
pub use self::node::{Node, NodeLocation};
pub use self::node_str::NodeStr;
pub use self::parser::{parse_mixed, Parser, SourceFile, SourceMap};
pub use self::schema::Schema;
pub use self::validation::{Diagnostics, GraphQLError, GraphQLLocation};

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

        Self { db }
    }
}
