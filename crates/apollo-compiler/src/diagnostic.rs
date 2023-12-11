//! Pretty-printable diagnostic reports for custom errors that reference GraphQL documents.
//!
//! The [`Diagnostic`] type wraps errors that implement [`ToDiagnostic`].
use crate::validation::FileId;
use crate::validation::NodeLocation;
use crate::SourceFile;
use crate::SourceMap;
use ariadne::ColorGenerator;
use ariadne::ReportKind;
use std::fmt;
use std::io;
use std::ops::Range;
use std::sync::Arc;
use std::sync::OnceLock;

#[cfg(doc)]
use crate::{ExecutableDocument, Schema};

type MappedSpan = (FileId, Range<usize>);

/// Translate a byte-offset location into a char-offset location for use with ariadne.
fn map_span(sources: &SourceMap, location: NodeLocation) -> Option<MappedSpan> {
    let source = sources.get(&location.file_id)?;
    let mapped_source = source.mapped_source();
    let start = mapped_source.map_index(location.offset());
    let end = mapped_source.map_index(location.end_offset());
    Some((location.file_id, start..end))
}

/// A diagnostic report that can be printed to a CLI with pretty colours and labeled lines of
/// GraphQL source code.
///
/// Custom errors can use this in their `Display` or `Debug` implementations to build a report and
/// then write it out with [`fmt`].
///
/// [`fmt`]: DiagnosticReport::fmt
pub struct DiagnosticReport {
    sources: SourceMap,
    colors: ColorGenerator,
    report: ariadne::ReportBuilder<'static, MappedSpan>,
}

impl DiagnosticReport {
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

    /// Write the report to a [`Write`], with colors.
    ///
    /// If colored output is not desired, consider wrapping the [`Write`] with [anstream].
    ///
    /// [`Write`]: std::io::Write
    pub fn write(self, w: impl std::io::Write) -> std::io::Result<()> {
        let report = self.report.finish();
        report.write(Cache(&self.sources), w)
    }

    /// Write the report to a [`fmt::Formatter`]. Alternate formatting disables colors.
    pub fn fmt(self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        struct StripColorAdaptor<'a, 'b> {
            f: &'a mut fmt::Formatter<'b>,
            strip: anstream::adapter::StripBytes,
        }
        impl io::Write for StripColorAdaptor<'_, '_> {
            fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
                for printable in self.strip.strip_next(buf) {
                    let s = std::str::from_utf8(printable).map_err(|_| io::ErrorKind::Other)?;
                    self.f.write_str(s).map_err(|_| io::ErrorKind::Other)?;
                }

                Ok(buf.len())
            }

            fn flush(&mut self) -> io::Result<()> {
                Ok(())
            }
        }

        struct ColorAdaptor<'a, 'b> {
            f: &'a mut fmt::Formatter<'b>,
        }
        impl io::Write for ColorAdaptor<'_, '_> {
            fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
                let s = std::str::from_utf8(buf).map_err(|_| io::ErrorKind::Other)?;
                self.f.write_str(s).map_err(|_| io::ErrorKind::Other)?;
                Ok(buf.len())
            }

            fn flush(&mut self) -> io::Result<()> {
                Ok(())
            }
        }

        if f.alternate() {
            self.write(StripColorAdaptor {
                f,
                strip: Default::default(),
            })
            .map_err(|_| fmt::Error)
        } else {
            self.write(ColorAdaptor { f }).map_err(|_| fmt::Error)
        }
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

/// A pretty-printable diagnostic.
#[derive(Debug)]
pub struct Diagnostic<T> {
    pub sources: SourceMap,
    pub location: Option<NodeLocation>,
    pub error: T,
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

    fn report(&self, report: &mut DiagnosticReport) {
        ToDiagnostic::report(*self, report)
    }
}

impl<T: ToDiagnostic> Diagnostic<T> {
    /// Write the report to a [`Write`], with colors.
    ///
    /// If colored output is not desired, consider wrapping the [`Write`] with [`anstream`].
    ///
    /// [`Write`]: std::io::Write
    pub fn write(&self, w: impl std::io::Write) -> std::io::Result<()> {
        let mut report = DiagnosticReport::builder(self.sources.clone(), self.error.location());
        self.error.report(&mut report);
        report.write(w)
    }
}

impl<T: ToDiagnostic> fmt::Display for Diagnostic<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        struct StripColorAdaptor<'a, 'b> {
            f: &'a mut fmt::Formatter<'b>,
            strip: anstream::adapter::StripBytes,
        }
        impl io::Write for StripColorAdaptor<'_, '_> {
            fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
                for printable in self.strip.strip_next(buf) {
                    let s = std::str::from_utf8(printable).map_err(|_| io::ErrorKind::Other)?;
                    self.f.write_str(s).map_err(|_| io::ErrorKind::Other)?;
                }

                Ok(buf.len())
            }

            fn flush(&mut self) -> io::Result<()> {
                Ok(())
            }
        }

        struct ColorAdaptor<'a, 'b> {
            f: &'a mut fmt::Formatter<'b>,
        }
        impl io::Write for ColorAdaptor<'_, '_> {
            fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
                let s = std::str::from_utf8(buf).map_err(|_| io::ErrorKind::Other)?;
                self.f.write_str(s).map_err(|_| io::ErrorKind::Other)?;
                Ok(buf.len())
            }

            fn flush(&mut self) -> io::Result<()> {
                Ok(())
            }
        }

        if f.alternate() {
            self.write(StripColorAdaptor {
                f,
                strip: Default::default(),
            })
            .map_err(|_| fmt::Error)
        } else {
            self.write(ColorAdaptor { f }).map_err(|_| fmt::Error)
        }
    }
}

/// Trait for pretty-printing custom error types.
pub trait ToDiagnostic {
    /// Return the main location for this error. May be `None` if a location doesn't make sense for
    /// the particular error.
    fn location(&self) -> Option<NodeLocation>;

    /// Create a diagnostic report based on this error type.
    fn report(&self, report: &mut DiagnosticReport) -> ();

    /// Returns a pretty-printable diagnostic.
    fn to_diagnostic(self, sources: &SourceMap) -> Diagnostic<Self>
    where
        Self: Sized,
    {
        Diagnostic {
            sources: sources.clone(),
            location: self.location(),
            error: self,
        }
    }
}

/// Extension trait adding a convenience method to Results to turn the error branch into a pretty-printable [`Diagnostic`].
pub trait ResultExt<T, E> {
    fn to_diagnostic(self, sources: &SourceMap) -> Result<T, Diagnostic<E>>;
}
impl<T, E: ToDiagnostic> ResultExt<T, E> for Result<T, E> {
    fn to_diagnostic(self, sources: &SourceMap) -> Result<T, Diagnostic<E>> {
        self.map_err(|error| error.to_diagnostic(sources))
    }
}
