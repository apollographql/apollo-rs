use crate::ast::Document;
use crate::Arc;
use crate::FileId;
use std::fmt;

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
    pub(crate) source_text: String,
    pub(crate) parse_errors: Vec<ParseError>,
}

#[derive(Clone, Eq, PartialEq, Hash)]
pub struct ParseError(pub(crate) apollo_parser::Error);

impl Parser {
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

    pub fn parse_ast(&mut self, source_text: impl Into<String>) -> Document {
        self.parse_with_file_id(source_text, FileId::new())
    }

    pub(crate) fn parse_with_file_id(
        &mut self,
        source_text: impl Into<String>,
        file_id: FileId,
    ) -> Document {
        let source_text = source_text.into();
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
            source_text,
            parse_errors: tree.errors().map(|err| ParseError(err.clone())).collect(),
        });
        Document::from_cst(tree.document(), file_id, source_file)
    }

    /// What level of recursion was reached during the last call to [`parse`][Self::parse].
    ///
    /// Collecting this on a corpus of documents can help decide
    /// how to set [`recursion_limit`][Self::recursion_limit].
    pub fn recursion_reached(&self) -> usize {
        self.recursion_reached
    }

    /// How many tokens were created during the last call to [`parse`][Self::parse].
    ///
    /// Collecting this on a corpus of documents can help decide
    /// how to set [`recursion_limit`][Self::token_limit].
    pub fn tokens_reached(&self) -> usize {
        self.tokens_reached
    }
}

impl SourceFile {
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
