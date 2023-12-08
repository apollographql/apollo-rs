use crate::ast;
use crate::ast::from_cst::Convert;
use crate::ast::Document;
use crate::executable;
use crate::schema::SchemaBuilder;
use crate::validation::Details;
use crate::validation::DiagnosticList;
use crate::validation::FileId;
use crate::validation::NodeLocation;
use crate::validation::Valid;
use crate::validation::WithErrors;
use crate::ExecutableDocument;
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
    pub(crate) source: OnceLock<MappedSource>,
}

pub type SourceMap = Arc<IndexMap<FileId, Arc<SourceFile>>>;

/// Translate byte offsets to ariadne's char offsets.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct MappedSource {
    ariadne: ariadne::Source,
    map: Vec<u32>,
}

/// Parse a schema and executable document from the given source text
/// containing a mixture of type system definitions and executable definitions.
/// and validate them.
/// This is mostly useful for unit tests.
///
/// `path` is the filesystem path (or arbitrary string) used in diagnostics
/// to identify this source file to users.
pub fn parse_mixed_validate(
    source_text: impl Into<String>,
    path: impl AsRef<Path>,
) -> Result<(Valid<Schema>, Valid<ExecutableDocument>), DiagnosticList> {
    Parser::new().parse_mixed_validate(source_text, path)
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
    pub fn parse_ast(
        &mut self,
        source_text: impl Into<String>,
        path: impl AsRef<Path>,
    ) -> Result<Document, WithErrors<Document>> {
        let mut errors = DiagnosticList::new(Default::default());
        let ast = self.parse_ast_inner(source_text, path, FileId::new(), &mut errors);
        errors.into_result_with(ast)
    }

    pub(crate) fn parse_ast_inner(
        &mut self,
        source_text: impl Into<String>,
        path: impl AsRef<Path>,
        file_id: FileId,
        errors: &mut DiagnosticList,
    ) -> Document {
        let tree = self.parse_common(
            source_text.into(),
            path.as_ref().to_owned(),
            file_id,
            errors,
            |parser| parser.parse(),
        );
        let sources = errors.sources.clone();
        Document::from_cst(tree.document(), file_id, sources)
    }

    pub(crate) fn parse_common<T: apollo_parser::cst::CstNode>(
        &mut self,
        source_text: String,
        path: PathBuf,
        file_id: FileId,
        errors: &mut DiagnosticList,
        parse: impl FnOnce(apollo_parser::Parser) -> apollo_parser::SyntaxTree<T>,
    ) -> apollo_parser::SyntaxTree<T> {
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
            source: OnceLock::new(),
        });
        Arc::make_mut(&mut errors.sources).insert(file_id, source_file);
        for parser_error in tree.errors() {
            // Silently skip parse errors at index beyond 4 GiB.
            // Rowan in apollo-parser might complain about files that large
            // before we get here anyway.
            let Ok(index) = parser_error.index().try_into() else {
                continue;
            };
            let Ok(len) = parser_error.data().len().try_into() else {
                continue;
            };
            let location = Some(NodeLocation {
                file_id,
                text_range: rowan::TextRange::at(index, len),
            });
            let details = if parser_error.is_limit() {
                Details::ParserLimit {
                    message: parser_error.message().to_owned(),
                }
            } else {
                Details::SyntaxError {
                    message: parser_error.message().to_owned(),
                }
            };
            errors.push(location, details)
        }
        tree
    }

    /// Parse the given source text as the sole input file of a schema.
    ///
    /// `path` is the filesystem path (or arbitrary string) used in diagnostics
    /// to identify this source file to users.
    ///
    /// To have multiple files contribute to a schema,
    /// use [`Schema::builder`] and [`Parser::parse_into_schema_builder`].
    pub fn parse_schema(
        &mut self,
        source_text: impl Into<String>,
        path: impl AsRef<Path>,
    ) -> Result<Schema, WithErrors<Schema>> {
        let mut builder = Schema::builder();
        self.parse_into_schema_builder(source_text, path, &mut builder);
        builder.build()
    }

    /// Parse the given source text as an additional input to a schema builder.
    ///
    /// `path` is the filesystem path (or arbitrary string) used in diagnostics
    /// to identify this source file to users.
    ///
    /// This can be used to build a schema from multiple source files.
    ///
    /// Errors (if any) are recorded in the builder and returned by [`SchemaBuilder::build`].
    pub fn parse_into_schema_builder(
        &mut self,
        source_text: impl Into<String>,
        path: impl AsRef<Path>,
        builder: &mut SchemaBuilder,
    ) {
        let ast = self.parse_ast_inner(source_text, path, FileId::new(), &mut builder.errors);
        let executable_definitions_are_errors = true;
        builder.add_ast_document_not_adding_sources(&ast, executable_definitions_are_errors);
    }

    /// Parse the given source text into an executable document, with the given schema.
    ///
    /// `path` is the filesystem path (or arbitrary string) used in diagnostics
    /// to identify this source file to users.
    pub fn parse_executable(
        &mut self,
        schema: &Valid<Schema>,
        source_text: impl Into<String>,
        path: impl AsRef<Path>,
    ) -> Result<ExecutableDocument, WithErrors<ExecutableDocument>> {
        let (document, errors) = self.parse_executable_inner(schema, source_text, path);
        errors.into_result_with(document)
    }

    pub(crate) fn parse_executable_inner(
        &mut self,
        schema: &Valid<Schema>,
        source_text: impl Into<String>,
        path: impl AsRef<Path>,
    ) -> (ExecutableDocument, DiagnosticList) {
        let mut errors = DiagnosticList::new(Default::default());
        let ast = self.parse_ast_inner(source_text, path, FileId::new(), &mut errors);
        let document = ast.to_executable_inner(schema, &mut errors);
        (document, errors)
    }

    /// Parse a schema and executable document from the given source text
    /// containing a mixture of type system definitions and executable definitions,
    /// and validate them.
    /// This is mostly useful for unit tests.
    ///
    /// `path` is the filesystem path (or arbitrary string) used in diagnostics
    /// to identify this source file to users.
    pub fn parse_mixed_validate(
        &mut self,
        source_text: impl Into<String>,
        path: impl AsRef<Path>,
    ) -> Result<(Valid<Schema>, Valid<ExecutableDocument>), DiagnosticList> {
        let mut builder = SchemaBuilder::new();
        let ast = self.parse_ast_inner(source_text, path, FileId::new(), &mut builder.errors);
        let executable_definitions_are_errors = false;
        let type_system_definitions_are_errors = false;
        builder.add_ast_document_not_adding_sources(&ast, executable_definitions_are_errors);
        let (schema, mut errors) = builder.build_inner();
        let executable = crate::executable::from_ast::document_from_ast(
            Some(&schema),
            &ast,
            &mut errors,
            type_system_definitions_are_errors,
        );
        crate::schema::validation::validate_schema(&mut errors, &schema);
        crate::executable::validation::validate_executable_document(
            &mut errors,
            &schema,
            &executable,
        );
        errors
            .into_result()
            .map(|()| (Valid(schema), Valid(executable)))
    }

    /// Parse the given source text as a selection set with optional outer brackets.
    ///
    /// `path` is the filesystem path (or arbitrary string) used in diagnostics
    /// to identify this source file to users.
    pub fn parse_field_set(
        &mut self,
        schema: &Valid<Schema>,
        type_name: ast::NamedType,
        source_text: impl Into<String>,
        path: impl AsRef<Path>,
    ) -> Result<executable::FieldSet, WithErrors<executable::FieldSet>> {
        let (field_set, errors) = self.parse_field_set_inner(schema, type_name, source_text, path);
        errors.into_result_with(field_set)
    }

    pub(crate) fn parse_field_set_inner(
        &mut self,
        schema: &Valid<Schema>,
        type_name: ast::NamedType,
        source_text: impl Into<String>,
        path: impl AsRef<Path>,
    ) -> (executable::FieldSet, DiagnosticList) {
        let file_id = FileId::new();
        let mut errors = DiagnosticList::new(Default::default());
        let tree = self.parse_common(
            source_text.into(),
            path.as_ref().to_owned(),
            file_id,
            &mut errors,
            |parser| parser.parse_selection_set(),
        );
        let ast = ast::from_cst::convert_selection_set(&tree.field_set(), file_id);
        let mut selection_set = executable::SelectionSet::new(type_name);
        let mut build_errors = executable::from_ast::BuildErrors {
            errors: &mut errors,
            path: executable::SelectionPath {
                nested_fields: Vec::new(),
                // 🤷
                root: executable::ExecutableDefinitionName::AnonymousOperation(
                    ast::OperationType::Query,
                ),
            },
        };
        selection_set.extend_from_ast(Some(schema), &mut build_errors, &ast);
        let field_set = executable::FieldSet {
            sources: errors.sources.clone(),
            selection_set,
        };
        (field_set, errors)
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
        let mut errors = DiagnosticList::new(Default::default());
        let file_id = FileId::new();
        let tree = self.parse_common(
            source_text.into(),
            path.as_ref().to_owned(),
            file_id,
            &mut errors,
            |parser| parser.parse_type(),
        );
        errors.into_result().map(|()| {
            tree.ty()
                .convert(file_id)
                .expect("conversion should be infallible if there were no syntax errors")
        })
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
}

impl std::fmt::Debug for SourceFile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self {
            path,
            source_text,
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
        debug_struct.finish()
    }
}
