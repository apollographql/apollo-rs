use crate::ast;
use crate::ast::from_cst::Convert;
use crate::ast::Document;
use crate::executable;
use crate::schema::SchemaBuilder;
use crate::validation::Details;
use crate::validation::DiagnosticList;
use crate::ExecutableDocument;
use crate::FileId;
use crate::NodeLocation;
use crate::Schema;
use indexmap::IndexMap;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::OnceLock;

/// Configuration for parsing an input string as GraphQL syntax
#[derive(Default, Debug, Clone)]
pub struct Parser {
    recursion_limit: Option<usize>,
    token_limit: Option<usize>,
    recursion_reached: usize,
    tokens_reached: usize,
}

/// Records for validation information about a file that was parsed
#[derive(Clone)]
pub struct SourceFile {
    pub(crate) path: PathBuf,
    pub(crate) source_text: String,
    pub(crate) parse_errors: Vec<apollo_parser::Error>,
    pub(crate) source: OnceLock<MappedSource>,
}

pub type SourceMap = Arc<IndexMap<FileId, Arc<SourceFile>>>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct MappedSource {
    ariadne: ariadne::Source,
    map: Vec<u32>,
}

/// Parse a schema and executable document from the given source text
/// containing a mixture of type system definitions and executable definitions.
/// This is mostly useful for unit tests.
///
/// `path` is the filesystem path (or arbitrary string) used in diagnostics
/// to identify this source file to users.
///
/// Parsing is fault-tolerant, so a schema and document are always returned.
/// TODO: document how to validate
pub fn parse_mixed(
    source_text: impl Into<String>,
    path: impl AsRef<Path>,
) -> (Schema, ExecutableDocument) {
    Parser::new().parse_mixed(source_text, path)
}

impl Parser {
    pub fn new() -> Self {
        Self::default()
    }

    /// Configure the recursion to use while parsing.
    pub fn recursion_limit(mut self, value: usize) -> Self {
        self.recursion_limit = Some(value);
        self
    }

    /// Configure the limit on the number of tokens to parse.
    /// If an input document is too big, parsing will be aborted.
    /// By default, there is no limit.
    pub fn token_limit(mut self, value: usize) -> Self {
        self.token_limit = Some(value);
        self
    }

    /// Parse the given source text into an AST document.
    ///
    /// `path` is the filesystem path (or arbitrary string) used in diagnostics
    /// to identify this source file to users.
    ///
    /// Parsing is fault-tolerant, so a document is always returned.
    /// In case of a parse error, [`Document::check_parse_errors`] will return relevant information
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
        let (tree, source_file) = self.parse_common(source_text, path, |parser| parser.parse());
        Document::from_cst(tree.document(), file_id, source_file)
    }

    pub(crate) fn parse_common<T: apollo_parser::cst::CstNode>(
        &mut self,
        source_text: String,
        path: PathBuf,
        parse: impl FnOnce(apollo_parser::Parser) -> apollo_parser::SyntaxTree<T>,
    ) -> (apollo_parser::SyntaxTree<T>, Arc<SourceFile>) {
        let mut parser = apollo_parser::Parser::new(&source_text);
        if let Some(value) = self.recursion_limit {
            parser = parser.recursion_limit(value)
        }
        if let Some(value) = self.token_limit {
            parser = parser.token_limit(value)
        }
        let tree = parse(parser);
        self.recursion_reached = tree.recursion_limit().high;
        self.tokens_reached = tree.token_limit().high;
        let source_file = Arc::new(SourceFile {
            path,
            source_text,
            parse_errors: tree.errors().cloned().collect(),
            source: OnceLock::new(),
        });
        (tree, source_file)
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

    /// Parse a schema and executable document from the given source text
    /// containing a mixture of type system definitions and executable definitions.
    /// This is mostly useful for unit tests.
    ///
    /// `path` is the filesystem path (or arbitrary string) used in diagnostics
    /// to identify this source file to users.
    ///
    /// Parsing is fault-tolerant, so a schema and document are always returned.
    /// TODO: document how to validate
    pub fn parse_mixed(
        &mut self,
        source_text: impl Into<String>,
        path: impl AsRef<Path>,
    ) -> (Schema, ExecutableDocument) {
        self.parse_ast(source_text, path).to_mixed()
    }

    /// Parse the given source text as a selection set with optional outer brackets.
    ///
    /// `path` is the filesystem path (or arbitrary string) used in diagnostics
    /// to identify this source file to users.
    ///
    /// Parsing is fault-tolerant, so a selection set node is always returned.
    /// TODO: document how to validate
    pub fn parse_field_set(
        &mut self,
        schema: &Schema,
        type_name: ast::NamedType,
        source_text: impl Into<String>,
        path: impl AsRef<Path>,
    ) -> executable::FieldSet {
        let (tree, source_file) =
            self.parse_common(source_text.into(), path.as_ref().to_owned(), |parser| {
                parser.parse_selection_set()
            });
        let file_id = FileId::new();
        let ast = ast::from_cst::convert_selection_set(&tree.field_set(), file_id);
        let mut selection_set = executable::SelectionSet::new(type_name);
        let mut build_errors = executable::from_ast::BuildErrors {
            errors: Vec::new(),
            path: executable::SelectionPath {
                nested_fields: Vec::new(),
                // 🤷
                root: executable::ExecutableDefinitionName::AnonymousOperation(
                    ast::OperationType::Query,
                ),
            },
        };
        selection_set.extend_from_ast(Some(schema), &mut build_errors, &ast);
        executable::FieldSet {
            sources: Arc::new([(file_id, source_file)].into()),
            build_errors: build_errors.errors,
            selection_set,
        }
    }

    /// Parse the given source text as a reference to a type.
    ///
    /// `path` is the filesystem path (or arbitrary string) used in diagnostics
    /// to identify this source file to users.
    pub fn parse_type(
        &mut self,
        source_text: impl Into<String>,
        path: impl AsRef<Path>,
    ) -> Result<ast::Type, DiagnosticList> {
        let (tree, source_file) =
            self.parse_common(source_text.into(), path.as_ref().to_owned(), |parser| {
                parser.parse_type()
            });
        let file_id = FileId::new();

        let sources: crate::SourceMap = Arc::new([(file_id, source_file)].into());
        let mut errors = DiagnosticList::new(None, sources.clone());
        for (file_id, source) in sources.iter() {
            source.validate_parse_errors(&mut errors, *file_id)
        }

        if errors.is_empty() {
            if let Some(ty) = tree.ty().convert(file_id) {
                return Ok(ty);
            }
            unreachable!("conversion is infallible if there were no syntax errors");
        } else {
            Err(errors)
        }
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

impl MappedSource {
    fn new(input: &str) -> Self {
        let ariadne = ariadne::Source::from(input);

        let mut map = vec![0; input.len() + 1];
        let mut char_index = 0;
        for (byte_index, _) in input.char_indices() {
            map[byte_index] = char_index;
            char_index += 1;
        }

        // Support 1 past the end of the string, for use in exclusive ranges.
        map[input.len()] = char_index;

        Self { ariadne, map }
    }

    pub(crate) fn map_index(&self, byte_index: usize) -> usize {
        self.map[byte_index] as usize
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

    pub(crate) fn ariadne(&self) -> &ariadne::Source {
        &self.mapped_source().ariadne
    }

    pub(crate) fn mapped_source(&self) -> &MappedSource {
        self.source
            .get_or_init(|| MappedSource::new(&self.source_text))
    }

    pub fn get_line_column(&self, index: usize) -> Option<(usize, usize)> {
        let char_index = self.mapped_source().map_index(index);
        let (_, line, column) = self.ariadne().get_offset_line(char_index)?;
        Some((line, column))
    }

    pub(crate) fn validate_parse_errors(&self, errors: &mut DiagnosticList, file_id: FileId) {
        for err in &self.parse_errors {
            // Silently skip parse errors at index beyond 4 GiB.
            // Rowan in apollo-parser might complain about files that large
            // before we get here anyway.
            let Ok(index) = err.index().try_into() else {
                continue;
            };
            let Ok(len) = err.data().len().try_into() else {
                continue;
            };
            let location = Some(NodeLocation {
                file_id,
                text_range: rowan::TextRange::at(index, len),
            });
            let details = if err.is_limit() {
                Details::ParserLimit {
                    message: err.message().to_owned(),
                }
            } else {
                Details::SyntaxError {
                    message: err.message().to_owned(),
                }
            };
            errors.push(location, details)
        }
    }
}

impl std::fmt::Debug for SourceFile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self {
            path,
            source_text,
            parse_errors,
            source: _, // Skipped: it’s a cache and would make debugging other things noisy
        } = self;
        let mut debug_struct = f.debug_struct("SourceFile");
        debug_struct.field("path", path);
        if path != std::path::Path::new("built_in.graphql") {
            debug_struct.field("source_text", source_text);
        } else {
            debug_struct.field(
                "source_text",
                &format_args!("include_str!(\"built_in.graphql\")"),
            );
        }
        debug_struct.field("parse_errors", parse_errors).finish()
    }
}
