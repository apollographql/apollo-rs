#![doc = include_str!("../README.md")]

#[macro_use]
mod macros;
mod arc;
pub mod ast;
pub mod database;
pub mod diagnostics;
pub mod executable;
mod node;
mod node_str;
mod parser;
pub mod schema;
pub mod validation;

use salsa::ParallelDatabase;
use std::path::Path;

pub use self::arc::Arc;
pub use self::database::{FileId, InputDatabase, ReprDatabase, RootDatabase, Source};
pub use self::diagnostics::ApolloDiagnostic;
pub use self::executable::ExecutableDocument;
pub use self::node::{Node, NodeLocation};
pub use self::node_str::NodeStr;
pub use self::parser::{parse_mixed, Parser, SourceFile};
use self::validation::ValidationDatabase;
pub use schema::Schema;

pub struct ApolloCompiler {
    pub db: RootDatabase,
}

/// A read-only, `Sync` snapshot of the database.
pub type Snapshot = salsa::Snapshot<RootDatabase>;

/// Apollo compiler creates a context around your GraphQL. It creates references
/// between various GraphQL types in scope.
///
/// ## Example
///
/// ```rust
/// use apollo_compiler::ApolloCompiler;
///
/// let input = r#"
///   interface Pet {
///     name: String
///   }
///
///   type Dog implements Pet {
///     name: String
///     nickname: String
///     barkVolume: Int
///   }
///
///   type Cat implements Pet {
///     name: String
///     nickname: String
///     meowVolume: Int
///   }
///
///   union CatOrDog = Cat | Dog
///
///   type Human {
///     name: String
///     pets: [Pet]
///   }
///
///   type Query {
///     human: Human
///   }
/// "#;
///
/// let mut compiler = ApolloCompiler::new();
/// compiler.add_type_system(input, "schema.graphql");
///
/// let diagnostics = compiler.validate();
/// for diagnostic in &diagnostics {
///     // this will pretty-print diagnostics using the miette crate.
///     println!("{}", diagnostic);
/// }
/// assert!(diagnostics.is_empty());
/// ```
#[allow(clippy::new_without_default)]
impl ApolloCompiler {
    /// Create a new instance of Apollo Compiler.
    pub fn new() -> Self {
        let mut db = RootDatabase::default();
        // TODO(@goto-bus-stop) can we make salsa fill in these defaults for us…?
        db.set_recursion_limit(None);
        db.set_token_limit(None);
        db.set_schema_input(None);
        db.set_source_files(vec![]);

        Self { db }
    }

    /// Create a new compiler with a pre-loaded GraphQL schema.
    ///
    /// This compiler can then only be used for executable documents that execute against that
    /// schema.
    pub fn from_schema(schema: Arc<Schema>) -> Self {
        let mut compiler = Self::new();
        compiler.db.set_schema_input(Some(schema));

        compiler
    }

    /// Configure the recursion limit to use during parsing.
    /// Recursion limit must be set prior to adding sources to the compiler.
    pub fn recursion_limit(mut self, limit: usize) -> Self {
        if !self.db.source_files().is_empty() {
            panic!(
                "There are already parsed files in the compiler. \
                 Setting recursion limit after files are parsed is not supported."
            );
        }
        self.db.set_recursion_limit(Some(limit));
        self
    }

    /// Configure the token limit to use during parsing.
    /// Token limit must be set prior to adding sources to the compiler.
    pub fn token_limit(mut self, limit: usize) -> Self {
        if !self.db.source_files().is_empty() {
            panic!(
                "There are already parsed files in the compiler. \
                 Setting token limit after files are parsed is not supported."
            );
        }
        self.db.set_token_limit(Some(limit));
        self
    }

    fn add_input(&mut self, source: Source) -> FileId {
        let file_id = FileId::new();
        let mut sources = self.db.source_files();
        sources.push(file_id);
        self.db.set_input(file_id, source);
        self.db.set_source_files(sources);

        file_id
    }

    /// Add a document with executable _and_ type system definitions and
    /// extensions to the compiler.
    ///
    /// The `path` argument is used to display diagnostics. If your GraphQL document
    /// doesn't come from a file, you can make up a name or provide the empty string.
    /// It does not need to be unique.
    ///
    /// Returns a `FileId` that you can use to update the source text of this document.
    pub fn add_document(&mut self, input: &str, path: impl AsRef<Path>) -> FileId {
        if self.db.schema_input().is_some() {
            panic!(
                "Having both string inputs and pre-computed inputs \
                 for type system definitions is not supported"
            )
        }
        let filename = path.as_ref().to_owned();
        self.add_input(Source::document(filename, input))
    }

    /// Add a document with type system definitions and extensions only to the compiler.
    ///
    /// The `path` argument is used to display diagnostics. If your GraphQL document
    /// doesn't come from a file, you can make up a name or provide the empty string.
    /// It does not need to be unique.
    ///
    /// Returns a `FileId` that you can use to update the source text of this document.
    pub fn add_type_system(&mut self, input: &str, path: impl AsRef<Path>) -> FileId {
        if self.db.schema_input().is_some() {
            panic!(
                "Having both string inputs and pre-computed inputs \
                 for type system definitions is not supported"
            )
        }
        let filename = path.as_ref().to_owned();
        self.add_input(Source::schema(filename, input))
    }

    /// Add a an executable document to the compiler.
    ///
    /// The `path` argument is used to display diagnostics. If your GraphQL document
    /// doesn't come from a file, you can make up a name or provide the empty string.
    /// It does not need to be unique.
    ///
    /// Returns a `FileId` that you can use to update the source text of this document.
    pub fn add_executable(&mut self, input: &str, path: impl AsRef<Path>) -> FileId {
        let filename = path.as_ref().to_owned();
        self.add_input(Source::executable(filename, input))
    }

    /// Update an existing GraphQL document with new source text. Queries that depend
    /// on this document will be recomputed.
    pub fn update_document(&mut self, file_id: FileId, input: &str) {
        let document = self.db.input(file_id);
        self.db.set_input(
            file_id,
            Source::document(document.filename().to_owned(), input),
        )
    }

    /// Update an existing GraphQL document with new source text. Queries that depend
    /// on this document will be recomputed.
    pub fn update_type_system(&mut self, file_id: FileId, input: &str) {
        let schema = self.db.input(file_id);
        self.db
            .set_input(file_id, Source::schema(schema.filename().to_owned(), input))
    }

    /// Update an existing GraphQL document with new source text. Queries that depend
    /// on this document will be recomputed.
    pub fn update_executable(&mut self, file_id: FileId, input: &str) {
        let executable = self.db.input(file_id);
        self.db.set_input(
            file_id,
            Source::executable(executable.filename().to_owned(), input),
        )
    }

    /// Get a snapshot of the current database.
    pub fn snapshot(&self) -> Snapshot {
        self.db.snapshot()
    }

    /// Validate your GraphQL input. Returns Diagnostics that you can pretty-print.
    ///
    /// ## Example
    /// ```rust
    /// use apollo_compiler::ApolloCompiler;
    /// let input = r#"
    /// type Query {
    ///   website: URL,
    ///   amount: Int
    /// }
    /// "#;
    ///
    /// let mut compiler = ApolloCompiler::new();
    /// compiler.add_document(input, "document.graphql");
    ///
    /// let diagnostics = compiler.validate();
    /// for diagnostic in &diagnostics {
    ///     println!("{}", diagnostic);
    /// }
    /// assert_eq!(diagnostics.len(), 1);
    /// ```
    pub fn validate(&self) -> Vec<ApolloDiagnostic> {
        self.db.validate()
    }
}
