//! Pretty-printable diagnostic reports for errors that reference GraphQL documents.
//!
//! # Usage
//! To use pretty-printing in custom errors, implement the [`ToDiagnostic`] trait.
//!
//! ```rust
//! use apollo_compiler::Schema;
//! use apollo_compiler::ast::Name;
//! use apollo_compiler::diagnostic::CliReport;
//! use apollo_compiler::diagnostic::Diagnostic;
//! use apollo_compiler::diagnostic::NodeLocation;
//! use apollo_compiler::diagnostic::ToDiagnostic;
//!
//! /// Error type for a small GraphQL schema linter.
//! #[derive(Debug, thiserror::Error)]
//! enum LintError {
//!     #[error("{name} should be PascalCase")]
//!     InvalidCase { name: Name },
//!     #[error("Missing @specifiedBy directive on scalar {name}")]
//!     NoSpecifiedBy {
//!         location: Option<NodeLocation>,
//!         name: Name,
//!     },
//! }
//!
//! impl ToDiagnostic for LintError {
//!     fn location(&self) -> Option<NodeLocation> {
//!         match self {
//!             LintError::InvalidCase { name } => name.location(),
//!             LintError::NoSpecifiedBy { location, .. } => *location,
//!         }
//!     }
//!
//!     fn report(&self, report: &mut CliReport) {
//!         match self {
//!             LintError::InvalidCase { name } => {
//!                 report.with_label_opt(name.location(), "should be PascalCase");
//!                 report.with_help(format!("Try using {}", to_pascal_case(name)));
//!             }
//!             LintError::NoSpecifiedBy { location, .. } => {
//!                 report.with_label_opt(*location, "scalar does not have a specification");
//!             }
//!         }
//!     }
//! }
//!
//! # fn to_pascal_case(name: &str) -> String { todo!() }
//! ```
//!
//! The [`Diagnostic`] type wraps errors that implement [`ToDiagnostic`] and provides
//! the pretty-printing functionality. [`ToDiagnostic::to_diagnostic`] returns a diagnostic
//! ready for formatting:
//!
//! ```rust
//! # use apollo_compiler::{Schema, diagnostic::{ToDiagnostic, NodeLocation, CliReport}};
//! # struct LintError {}
//! # impl ToDiagnostic for LintError {
//! #     fn location(&self) -> Option<NodeLocation> { None }
//! #     fn report(&self, _report: &mut CliReport) {}
//! # }
//! fn print_errors(schema: &Schema, errors: &[LintError]) {
//!     for error in errors {
//!         // Debug-formatting uses colors.
//!         eprintln!("{:?}", error.to_diagnostic(&schema.sources));
//!     }
//! }
//! ```
use crate::execution::GraphQLError;
use crate::execution::GraphQLLocation;
use crate::validation::FileId;
use crate::SourceFile;
use crate::SourceMap;
use ariadne::ColorGenerator;
use ariadne::ReportKind;
use std::fmt;
use std::io;
use std::ops::Range;
use std::sync::Arc;
use std::sync::OnceLock;

pub use crate::validation::NodeLocation;

#[cfg(doc)]
use crate::{ExecutableDocument, Schema};

/// A pretty-printable diagnostic.
pub struct Diagnostic<T> {
    pub sources: SourceMap,
    pub error: T,
}

/// A diagnostic report that can be printed to a CLI with pretty colors and labeled lines of
/// GraphQL source code.
///
/// Custom errors can use this in their `Display` or `Debug` implementations to build a report and
/// then write it out with [`fmt`].
///
/// [`fmt`]: CliReport::fmt
pub struct CliReport {
    sources: SourceMap,
    colors: ColorGenerator,
    report: ariadne::ReportBuilder<'static, MappedSpan>,
}

/// Indicate when to use ANSI colors for printing.
#[derive(Debug, Clone, Copy)]
pub enum Color {
    /// Do not use colors.
    Never,
    /// Use colors if stderr is a terminal.
    StderrIsTerminal,
}

/// Trait for pretty-printing custom error types.
pub trait ToDiagnostic {
    /// Return the main location for this error. May be `None` if a location doesn't make sense for
    /// the particular error.
    fn location(&self) -> Option<NodeLocation>;

    /// Fill in the report with messages and source code labels.
    fn report(&self, report: &mut CliReport);

    /// Returns a pretty-printable diagnostic.
    ///
    /// Provide a source map containing files that may be referenced by the diagnostic. Normally
    /// this comes from [`Schema::sources`] or [`ExecutableDocument::sources`].
    fn to_diagnostic(self, sources: &SourceMap) -> Diagnostic<Self>
    where
        Self: Sized,
    {
        Diagnostic {
            sources: sources.clone(),
            error: self,
        }
    }
}

/// Extension trait adding a convenience method to Results to turn the error branch into a pretty-printable [`Diagnostic`].
pub trait ResultExt<T, E> {
    fn to_diagnostic(self, sources: &SourceMap) -> Result<T, Diagnostic<E>>;
}

type MappedSpan = (FileId, Range<usize>);

/// Translate a byte-offset location into a char-offset location for use with ariadne.
fn map_span(sources: &SourceMap, location: NodeLocation) -> Option<MappedSpan> {
    let source = sources.get(&location.file_id)?;
    let mapped_source = source.mapped_source();
    let start = mapped_source.map_index(location.offset());
    let end = mapped_source.map_index(location.end_offset());
    Some((location.file_id, start..end))
}

/// Provide a [`std::io::Write`] API for a [`std::fmt::Formatter`].
struct WriteToFormatter<'a, 'b> {
    f: &'a mut fmt::Formatter<'b>,
}
impl io::Write for WriteToFormatter<'_, '_> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let s = std::str::from_utf8(buf).map_err(|_| io::ErrorKind::Other)?;
        self.f.write_str(s).map_err(|_| io::ErrorKind::Other)?;
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl CliReport {
    /// Returns a builder for creating diagnostic reports.
    ///
    /// Provide GraphQL source files and the main location for the diagnostic.
    /// Source files can be obtained from [`Schema::sources`] or [`ExecutableDocument::sources`].
    pub fn builder(sources: SourceMap, location: Option<NodeLocation>) -> Self {
        let (file_id, range) = location
            .and_then(|location| map_span(&sources, location))
            .unwrap_or((FileId::NONE, 0..0));
        Self {
            sources,
            colors: ColorGenerator::new(),
            report: ariadne::Report::build(ReportKind::Error, file_id, range.start),
        }
    }

    fn with_color(self, color: Color) -> Self {
        let enable_color = match color {
            Color::Never => false,
            // Rely on ariadne's `auto-color` feature, which uses `concolor` to enable colors
            // only if stderr is a terminal.
            Color::StderrIsTerminal => true,
        };
        let config = ariadne::Config::default().with_color(enable_color);
        Self {
            report: self.report.with_config(config),
            ..self
        }
    }

    /// Set the main message for the report.
    pub fn with_message(&mut self, message: impl ToString) {
        self.report.set_message(message);
    }

    /// Set the help message for the report, usually a suggestion on how to fix the error.
    pub fn with_help(&mut self, help: impl ToString) {
        self.report.set_help(help);
    }

    /// Set a note for the report, providing additional information that isn't related to a
    /// source location (when a label should be used).
    pub fn with_note(&mut self, note: impl ToString) {
        self.report.set_note(note);
    }

    /// Add a label at a given location. If the location is `None`, the message is discarded.
    pub fn with_label_opt(&mut self, location: Option<NodeLocation>, message: impl ToString) {
        if let Some(mapped_span) = location.and_then(|location| map_span(&self.sources, location)) {
            self.report.add_label(
                ariadne::Label::new(mapped_span)
                    .with_message(message)
                    .with_color(self.colors.next()),
            );
        }
    }

    /// Write the report to a [`Write`].
    ///
    /// [`Write`]: std::io::Write
    pub fn write(self, w: impl std::io::Write) -> std::io::Result<()> {
        let report = self.report.finish();
        report.write(Cache(&self.sources), w)
    }

    /// Write the report to a [`fmt::Formatter`].
    pub fn fmt(self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.write(WriteToFormatter { f }).map_err(|_| fmt::Error)
    }
}

struct Cache<'a>(&'a SourceMap);

impl ariadne::Cache<FileId> for Cache<'_> {
    fn fetch(&mut self, file_id: &FileId) -> Result<&ariadne::Source, Box<dyn fmt::Debug + '_>> {
        struct NotFound(FileId);
        impl fmt::Debug for NotFound {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "source file not found: {:?}", self.0)
            }
        }
        if let Some(source_file) = self.0.get(file_id) {
            Ok(source_file.ariadne())
        } else if *file_id == FileId::NONE || *file_id == FileId::HACK_TMP {
            static EMPTY: OnceLock<ariadne::Source> = OnceLock::new();
            Ok(EMPTY.get_or_init(|| ariadne::Source::from("")))
        } else {
            Err(Box::new(NotFound(*file_id)))
        }
    }

    fn display<'a>(&self, file_id: &'a FileId) -> Option<Box<dyn fmt::Display + 'a>> {
        if *file_id != FileId::NONE {
            struct Path(Arc<SourceFile>);
            impl fmt::Display for Path {
                fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                    self.0.path().display().fmt(f)
                }
            }
            let source_file = self.0.get(file_id)?;
            Some(Box::new(Path(source_file.clone())))
        } else {
            struct NoSourceFile;
            impl fmt::Display for NoSourceFile {
                fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                    f.write_str("(no source file)")
                }
            }
            Some(Box::new(NoSourceFile))
        }
    }
}

impl<T> std::ops::Deref for Diagnostic<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.error
    }
}

impl<T: std::error::Error + ToDiagnostic> std::error::Error for Diagnostic<T> {}

impl<T: ToDiagnostic> ToDiagnostic for &T {
    fn location(&self) -> Option<NodeLocation> {
        ToDiagnostic::location(*self)
    }

    fn report(&self, report: &mut CliReport) {
        ToDiagnostic::report(*self, report)
    }
}

impl<T: ToDiagnostic> Diagnostic<T> {
    /// Get the line and column number where this diagnostic was raised.
    pub fn get_line_column(&self) -> Option<GraphQLLocation> {
        GraphQLLocation::from_node(&self.sources, self.error.location())
    }

    /// Get a [`serde`]-serializable version of the current diagnostic. The shape is compatible
    /// with the JSON error shape described in [the GraphQL spec].
    ///
    /// [the GraphQL spec]: https://spec.graphql.org/draft/#sec-Errors
    pub fn to_json(&self) -> GraphQLError
    where
        T: ToString,
    {
        GraphQLError::new(self.error.to_string(), self.error.location(), &self.sources)
    }

    /// Produce the diagnostic report, optionally with colors for the CLI.
    fn report(&self, color: Color) -> CliReport {
        let mut report =
            CliReport::builder(self.sources.clone(), self.error.location()).with_color(color);
        self.error.report(&mut report);
        report
    }

    /// Pretty-print the diagnostic to a [`Write`].
    ///
    /// [`Write`]: std::io::Write
    pub fn write(&self, color: Color, w: impl std::io::Write) -> std::io::Result<()> {
        self.report(color).write(w)
    }
}

impl<T: ToDiagnostic> fmt::Debug for Diagnostic<T> {
    /// Pretty-format the diagnostic, with colors for the CLI.
    ///
    /// The debug formatting expects to be written to stderr and ANSI colors are used if stderr is
    /// a terminal.
    ///
    /// To output *without* colors, format with `Display`: `format!("{diagnostic}")`
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.report(Color::StderrIsTerminal).fmt(f)
    }
}

impl<T: ToDiagnostic> fmt::Display for Diagnostic<T> {
    /// Pretty-format the diagnostic without colors.
    ///
    /// To output *with* colors, format with `Debug`: `eprintln!("{diagnostic:?}")`
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.report(Color::Never).fmt(f)
    }
}

impl<T, E: ToDiagnostic> ResultExt<T, E> for Result<T, E> {
    /// Turns the `Err()` branch into a pretty-printable diagnostic.
    fn to_diagnostic(self, sources: &SourceMap) -> Result<T, Diagnostic<E>> {
        self.map_err(|error| error.to_diagnostic(sources))
    }
}
