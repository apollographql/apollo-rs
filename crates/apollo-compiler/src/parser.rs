//! APIs related to parsing `&str` inputs as GraphQL syntax.
//!
//! This module typically does not need to be imported directly.
//! If the default parser configuration is adequate, use constructors such as:
//!
//! * [`ast::Document::parse`]
//! * [`Schema::parse`]
//! * [`Schema::parse_and_validate`]
//! * [`ExecutableDocument::parse`]
//! * [`ExecutableDocument::parse_and_validate`]
//!
//! If not, create a [`Parser`] and use its builder methods to change configuration.

use crate::ast;
use crate::ast::from_cst::Convert;
use crate::ast::Document;
use crate::collections::IndexMap;
use crate::executable;
use crate::schema::SchemaBuilder;
use crate::validation::Details;
use crate::validation::DiagnosticList;
use crate::validation::Valid;
use crate::validation::WithErrors;
use crate::ExecutableDocument;
use crate::Schema;
use apollo_parser::SyntaxNode;
use rowan::TextRange;
use serde::Deserialize;
use serde::Serialize;
use std::num::NonZeroU64;
use std::ops::Range;
use std::path::Path;
use std::path::PathBuf;
use std::sync::atomic;
use std::sync::atomic::AtomicU64;
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
    pub(crate) source: OnceLock<ariadne::Source>,
}

/// A map of source files relevant to a given document
pub type SourceMap = Arc<IndexMap<FileId, Arc<SourceFile>>>;

/// Integer identifier for a parsed source file.
///
/// Used internally to support validating for example a schema built from multiple source files,
/// and having diagnostics point to relevant sources.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct FileId {
    id: NonZeroU64,
}

#[derive(Copy, Clone)]
pub(crate) struct TaggedFileId {
    tag_and_id: NonZeroU64,
}

/// The source location of a parsed node:
/// file ID and text range (start and end byte offsets) within that file.
#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct SourceSpan {
    pub(crate) file_id: FileId,
    pub(crate) text_range: TextRange,
}

/// A line number and column number within a GraphQL document.
#[derive(Clone, Copy, Hash, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LineColumn {
    /// The line number for this location, starting at 1 for the first line.
    pub line: usize,
    /// The column number for this location, starting at 1 and counting characters (Unicode Scalar
    /// Values) like [`str::chars`].
    pub column: usize,
}

impl std::fmt::Debug for LineColumn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.line, self.column)
    }
}

impl Parser {
    /// Create a `Parser` with default configuration.
    /// Use other methods to change the configuration.
    pub fn new() -> Self {
        Self::default()
    }

    /// Configure the recursion limit to use while parsing.
    ///
    /// This protects against stack overflow.
    /// If unset, use [`apollo-parser`][apollo_parser]’s default limit.
    /// The exact meaning is unspecified,
    /// but for GraphQL constructs like selection sets whose syntax can be nested,
    /// the nesting level encountered during parsing counts towards this limit.
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
            let location = Some(SourceSpan {
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
    #[allow(clippy::result_large_err)] // Typically not called very often
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
    #[allow(clippy::result_large_err)] // Typically not called very often
    pub fn parse_executable(
        &mut self,
        schema: &Valid<Schema>,
        source_text: impl Into<String>,
        path: impl AsRef<Path>,
    ) -> Result<ExecutableDocument, WithErrors<ExecutableDocument>> {
        let (document, errors) = self.parse_executable_inner(schema, source_text, path);
        errors.into_result_with(document)
    }

    /// Parse the given source text as an additional input to an executable document builder.
    ///
    /// `path` is the filesystem path (or arbitrary string) used in diagnostics
    /// to identify this source file to users.
    ///
    /// This can be used to build an executable document from multiple source files.
    ///
    /// # Arguments
    ///
    /// * `schema` - Optional schema for type checking. If provided, operations and fragments
    ///   will be validated against the schema while building.
    /// * `source_text` - The GraphQL source text to parse
    /// * `path` - Path used in diagnostics to identify this source file
    /// * `builder` - The builder to add parsed definitions to
    ///
    /// # Example
    ///
    /// ```rust
    /// use apollo_compiler::{Schema, ExecutableDocument};
    /// use apollo_compiler::parser::Parser;
    /// use apollo_compiler::validation::DiagnosticList;
    /// # let schema_src = "type Query { user: User, post: Post } type User { id: ID } type Post { title: String }";
    /// # let schema = Schema::parse_and_validate(schema_src, "schema.graphql").unwrap();
    ///
    /// let mut errors = DiagnosticList::new(Default::default());
    /// let mut builder = ExecutableDocument::builder(Some(&schema), &mut errors);
    /// let mut parser = Parser::new();
    ///
    /// parser.parse_into_executable_builder(
    ///     "query GetUser { user { id } }",
    ///     "query1.graphql",
    ///     &mut builder,
    /// );
    /// parser.parse_into_executable_builder(
    ///     "query GetPost { post { title } }",
    ///     "query2.graphql",
    ///     &mut builder,
    /// );
    ///
    /// let document = builder.build();
    /// assert!(errors.is_empty());
    /// ```
    ///
    /// Errors (if any) are recorded in the builder and returned by
    /// [`ExecutableDocumentBuilder::build`].
    pub fn parse_into_executable_builder(
        &mut self,
        source_text: impl Into<String>,
        path: impl AsRef<Path>,
        builder: &mut executable::ExecutableDocumentBuilder,
    ) {
        let ast = self.parse_ast_inner(source_text, path, FileId::new(), &mut builder.errors);
        let type_system_definitions_are_errors = true;
        builder.add_ast_document(&ast, type_system_definitions_are_errors);
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
        let (mut schema, mut errors) = builder.build_inner();
        let executable = crate::executable::from_ast::document_from_ast(
            Some(&schema),
            &ast,
            &mut errors,
            type_system_definitions_are_errors,
        );
        crate::schema::validation::validate_schema(&mut errors, &mut schema);
        crate::executable::validation::validate_executable_document(
            &mut errors,
            &schema,
            &executable,
        );
        errors
            .into_result()
            .map(|()| (Valid(schema), Valid(executable)))
    }

    /// Parse the given source text (e.g. `field_1 field_2 { field_2_1 }`
    /// as a selection set with optional outer brackets.
    ///
    /// This is the syntax of the string argument to some Apollo Federation directives.
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

    /// Parse the given source text (e.g. `[Foo!]!`) as a reference to a GraphQL type.
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
    /// how to set [`token_limit`][Self::token_limit].
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

    pub(crate) fn ariadne(&self) -> &ariadne::Source {
        self.source.get_or_init(|| {
            // FIXME This string copy is not ideal, but changing to a reference counted string affects
            // public API
            ariadne::Source::from(self.source_text.clone())
        })
    }

    /// Get [`LineColumn`] for the given 0-indexed UTF-8 byte `offset` from the start of the file.
    ///
    /// Returns None if the offset is out of bounds.
    pub fn get_line_column(&self, offset: usize) -> Option<LineColumn> {
        let (_, zero_indexed_line, zero_indexed_column) = self.ariadne().get_byte_line(offset)?;
        Some(LineColumn {
            line: zero_indexed_line + 1,
            column: zero_indexed_column + 1,
        })
    }

    /// Get starting and ending [`LineColumn`]s for the given `range` 0-indexed UTF-8 byte offsets.
    ///
    /// Returns `None` if either offset is out of bounds.
    pub fn get_line_column_range(&self, range: Range<usize>) -> Option<Range<LineColumn>> {
        let start = self.get_line_column(range.start)?;
        let end = self.get_line_column(range.end)?;
        Some(start..end)
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

impl std::fmt::Debug for FileId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.id.fmt(f)
    }
}

/// The next file ID to use. This is global so file IDs do not conflict between different compiler
/// instances.
static NEXT: AtomicU64 = AtomicU64::new(INITIAL);
static INITIAL: u64 = 3;

const TAG: u64 = 1 << 63;
const ID_MASK: u64 = !TAG;

#[allow(clippy::assertions_on_constants)]
const _: () = {
    assert!(TAG == 0x8000_0000_0000_0000);
    assert!(ID_MASK == 0x7FFF_FFFF_FFFF_FFFF);
};

impl FileId {
    /// The ID of the file implicitly added to type systems, for built-in scalars and introspection types
    pub const BUILT_IN: Self = Self::const_new(1);

    /// Passed to Ariadne to create a report without a location
    pub(crate) const NONE: Self = Self::const_new(2);

    // Returning a different value every time does not sound like good `impl Default`
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        loop {
            let id = NEXT.fetch_add(1, atomic::Ordering::AcqRel);
            if id & TAG == 0 {
                return Self {
                    id: NonZeroU64::new(id).unwrap(),
                };
            } else {
                // Overflowing 63 bits is unlikely, but if it somehow happens
                // reset the counter and try again.
                //
                // `TaggedFileId` behaving incorrectly would be a memory safety issue,
                // whereas a file ID collision “merely” causes
                // diagnostics to print the wrong file name and source context.
                Self::reset()
            }
        }
    }

    /// Reset file ID counter back to its initial value, used to get consistent results in tests.
    ///
    /// All tests in the process must use `#[serial_test::serial]`
    #[doc(hidden)]
    pub fn reset() {
        NEXT.store(INITIAL, atomic::Ordering::Release)
    }

    const fn const_new(id: u64) -> Self {
        assert!(id & ID_MASK == id);
        // TODO: use unwrap() when const-stable https://github.com/rust-lang/rust/issues/67441
        if let Some(id) = NonZeroU64::new(id) {
            Self { id }
        } else {
            panic!()
        }
    }
}

impl TaggedFileId {
    pub(crate) const fn pack(tag: bool, id: FileId) -> Self {
        debug_assert!((id.id.get() & TAG) == 0);
        let tag_and_id = if tag {
            let packed = id.id.get() | TAG;
            // SAFETY: `id.id` was non-zero, so setting an additional bit is still non-zero
            unsafe { NonZeroU64::new_unchecked(packed) }
        } else {
            id.id
        };
        Self { tag_and_id }
    }

    pub(crate) fn tag(self) -> bool {
        (self.tag_and_id.get() & TAG) != 0
    }

    pub(crate) fn file_id(self) -> FileId {
        let unpacked = self.tag_and_id.get() & ID_MASK;
        // SAFETY: `unpacked` has the same value as `id: FileId` did in `pack()`, which is non-zero
        let id = unsafe { NonZeroU64::new_unchecked(unpacked) };
        FileId { id }
    }
}

impl SourceSpan {
    pub(crate) fn new(file_id: FileId, node: &'_ SyntaxNode) -> Self {
        Self {
            file_id,
            text_range: node.text_range(),
        }
    }

    /// Returns the file ID for this location
    pub fn file_id(&self) -> FileId {
        self.file_id
    }

    /// Returns the offset from the start of the file to the start of the range, in UTF-8 bytes
    pub fn offset(&self) -> usize {
        self.text_range.start().into()
    }

    /// Returns the offset from the start of the file to the end of the range, in UTF-8 bytes
    pub fn end_offset(&self) -> usize {
        self.text_range.end().into()
    }

    /// Returns the length of the range, in UTF-8 bytes
    pub fn node_len(&self) -> usize {
        self.text_range.len().into()
    }

    /// Best effort at making a location with the given start and end
    pub fn recompose(start_of: Option<Self>, end_of: Option<Self>) -> Option<Self> {
        match (start_of, end_of) {
            (None, None) => None,
            (None, single @ Some(_)) | (single @ Some(_), None) => single,
            (Some(start), Some(end)) => {
                if start.file_id != end.file_id {
                    // Pick one aribtrarily
                    return Some(end);
                }
                Some(SourceSpan {
                    file_id: start.file_id,
                    text_range: TextRange::new(start.text_range.start(), end.text_range.end()),
                })
            }
        }
    }

    /// The line and column numbers of [`Self::offset`]
    pub fn line_column(&self, sources: &SourceMap) -> Option<LineColumn> {
        let source = sources.get(&self.file_id)?;
        source.get_line_column(self.offset())
    }

    /// The line and column numbers of the range from [`Self::offset`] to [`Self::end_offset`]
    /// inclusive.
    pub fn line_column_range(&self, sources: &SourceMap) -> Option<Range<LineColumn>> {
        let source = sources.get(&self.file_id)?;
        source.get_line_column_range(self.offset()..self.end_offset())
    }
}

impl std::fmt::Debug for SourceSpan {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}..{} @{:?}",
            self.offset(),
            self.end_offset(),
            self.file_id,
        )
    }
}
