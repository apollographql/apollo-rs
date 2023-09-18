use crate::ast::Document;
use crate::schema::SchemaBuilder;
use crate::Arc;
use crate::ExecutableDocument;
use crate::FileId;
use crate::Schema;
use std::fmt;
use std::path::Path;
use std::path::PathBuf;

/// Configuration for parsing an input string as GraphQL syntax
#[derive(Default, Debug, Clone)]
pub struct Parser {
    recursion_limit: Option<usize>,
    token_limit: Option<usize>,
    recursion_reached: usize,
    tokens_reached: usize,
}

#[derive(Debug)]
pub struct SourceFile {
    pub(crate) path: PathBuf,
    pub(crate) source_text: String,
    pub(crate) parse_errors: Vec<ParseError>,
}

#[derive(Clone, Eq, PartialEq, Hash)]
pub struct ParseError(pub(crate) apollo_parser::Error);

impl Parser {
    pub fn new() -> Self {
        Self::default()
    }

    /// Configure the recursion to use while parsing.
    pub fn recursion_limit(&mut self, value: usize) -> &mut Self {
        self.recursion_limit = Some(value);
        self
    }

    /// Configure the limit on the number of tokens to parse.
    /// If an input document is too big, parsing will be aborted.
    /// By default, there is no limit.
    pub fn token_limit(&mut self, value: usize) -> &mut Self {
        self.token_limit = Some(value);
        self
    }

    /// Parse the given source text into an AST document.
    ///
    /// `path` is the filesystem path (or arbitrary string) used in diagnostics
    /// to identify this source file to users.
    ///
    /// Parsing is fault-tolerant, so a document is always returned.
    /// In case of a parse error, [`Document::parse_errors`] will return relevant information
    /// and some nodes may be missing in the built document.
    pub fn parse_ast(
        &mut self,
        source_text: impl Into<String>,
        path: impl AsRef<Path>,
    ) -> Document {
        self.parse_with_file_id(source_text.into(), path.as_ref().to_owned(), FileId::new())
    }

    pub(crate) fn parse_with_file_id(
        &mut self,
        source_text: String,
        path: PathBuf,
        file_id: FileId,
    ) -> Document {
        let mut parser = apollo_parser::Parser::new(&source_text);
        if let Some(value) = self.recursion_limit {
            parser = parser.recursion_limit(value)
        }
        if let Some(value) = self.token_limit {
            parser = parser.token_limit(value)
        }
        let tree = parser.parse();
        self.recursion_reached = tree.recursion_limit().high;
        self.tokens_reached = tree.token_limit().high;
        let source_file = Arc::new(SourceFile {
            path,
            source_text,
            parse_errors: tree.errors().map(|err| ParseError(err.clone())).collect(),
        });
        Document::from_cst(tree.document(), file_id, source_file)
    }

    /// Parse the given source text as the sole input file of a schema.
    ///
    /// `path` is the filesystem path (or arbitrary string) used in diagnostics
    /// to identify this source file to users.
    ///
    /// To have multiple files contribute to a schema,
    /// use [`Schema::builder`] and [`Parser::parse_into_schema_builder`].
    ///
    /// Parsing is fault-tolerant, so a schema is always returned.
    /// TODO: document how to validate
    pub fn parse_schema(
        &mut self,
        source_text: impl Into<String>,
        path: impl AsRef<Path>,
    ) -> Schema {
        self.parse_ast(source_text, path).to_schema()
    }

    /// Parse the given source text as an additional input to a schema builder.
    ///
    /// `path` is the filesystem path (or arbitrary string) used in diagnostics
    /// to identify this source file to users.
    ///
    /// This can be used to build a schema from multiple source files.
    ///
    /// Parsing is fault-tolerant, so this is infallible.
    /// TODO: document how to validate.
    pub fn parse_into_schema_builder(
        &mut self,
        source_text: impl Into<String>,
        path: impl AsRef<Path>,
        builder: &mut SchemaBuilder,
    ) {
        self.parse_ast(source_text, path).to_schema_builder(builder)
    }

    /// Parse the given source text into an executable document, with the given schema.
    ///
    /// `path` is the filesystem path (or arbitrary string) used in diagnostics
    /// to identify this source file to users.
    ///
    /// Parsing is fault-tolerant, so a document is always returned.
    /// TODO: document how to validate
    pub fn parse_executable(
        &mut self,
        schema: &Schema,
        source_text: impl Into<String>,
        path: impl AsRef<Path>,
    ) -> ExecutableDocument {
        self.parse_ast(source_text, path).to_executable(schema)
    }

    /// What level of recursion was reached during the last call to a `parse_*` method.
    ///
    /// Collecting this on a corpus of documents can help decide
    /// how to set [`recursion_limit`][Self::recursion_limit].
    pub fn recursion_reached(&self) -> usize {
        self.recursion_reached
    }

    /// How many tokens were created during the last call to a `parse_*` method.
    ///
    /// Collecting this on a corpus of documents can help decide
    /// how to set [`recursion_limit`][Self::token_limit].
    pub fn tokens_reached(&self) -> usize {
        self.tokens_reached
    }
}

impl SourceFile {
    /// The filesystem path (or arbitrary string) used in diagnostics
    /// to identify this source file to users.
    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn source_text(&self) -> &str {
        &self.source_text
    }

    pub fn parse_errors(&self) -> &[ParseError] {
        &self.parse_errors
    }
}

impl fmt::Debug for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.0, f)
    }
}
