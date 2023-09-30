mod validation_db;

mod argument;
mod directive;
mod enum_;
mod extension;
mod field;
mod fragment;
mod input_object;
mod interface;
mod object;
mod operation;
mod scalar;
mod schema;
mod selection;
mod union_;
mod value;
mod variable;

use crate::Arc;
use crate::FileId;
use crate::NodeLocation;
use crate::NodeStr;
use crate::SourceFile;
use indexmap::IndexMap;
use indexmap::IndexSet;
use std::cell::RefCell;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::fmt;
use std::io;
pub use validation_db::{ValidationDatabase, ValidationStorage};

pub struct Diagnostics(Box<DiagnosticsBoxed>);

/// Box indirection to avoid large `Err` values:
/// <https://rust-lang.github.io/rust-clippy/master/index.html#result_large_err>
struct DiagnosticsBoxed {
    source_cache: RefCell<SourceCache>,
    errors: Vec<Error>,
}

struct SourceCache {
    sources: IndexMap<FileId, Arc<SourceFile>>,
    cache: HashMap<FileId, ariadne::Source>,
}

struct Error {
    location: Option<NodeLocation>,
    details: Details,
}

#[derive(thiserror::Error, Debug)]
pub(crate) enum Details {
    #[error("{message}")]
    ParserLimit { message: String },
    #[error("syntax error: {message}")]
    SyntaxError { message: String },
}

impl Error {
    fn report(&self, color: bool) -> ariadne::Report<'static, NodeLocation> {
        let (id, offset) = if let Some(location) = self.location {
            (location.file_id(), location.offset())
        } else {
            (FileId::NONE, 0)
        };

        let mut report = ariadne::Report::build::<FileId>(ariadne::ReportKind::Error, id, offset)
            .with_config(ariadne::Config::default().with_color(color))
            .with_message(&self.details);
        let mut colors = ariadne::ColorGenerator::new();
        macro_rules! opt_label {
            ($location: expr, $message: expr) => {
                if let Some(location) = $location {
                    report.add_label(
                        ariadne::Label::new(location)
                            .with_message($message)
                            .with_color(colors.next()),
                    )
                }
            };
            ($message: expr) => {
                opt_label!(self.location, $message)
            };
        }
        match &self.details {
            Details::ParserLimit { message, .. } => opt_label!(message),
            Details::SyntaxError { message, .. } => opt_label!(message),
        }
        report.finish()
    }
}

impl Diagnostics {
    pub(crate) fn new(sources: IndexMap<FileId, Arc<SourceFile>>) -> Self {
        Self(Box::new(DiagnosticsBoxed {
            errors: Vec::new(),
            source_cache: RefCell::new(SourceCache {
                sources,
                cache: HashMap::new(),
            }),
        }))
    }

    pub(crate) fn push(&mut self, location: Option<NodeLocation>, details: Details) {
        self.0.errors.push(Error { location, details })
    }

    pub(crate) fn into_result(mut self) -> Result<(), Self> {
        if self.0.errors.is_empty() {
            Ok(())
        } else {
            self.0
                .errors
                .sort_by_key(|err| err.location.map(|loc| (loc.file_id(), loc.offset())));
            Err(self)
        }
    }
}

/// Use alternate formatting to disable colors: `format!("{validation_errors:#}")`
impl fmt::Display for Diagnostics {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        struct Adaptor<'a, 'b>(&'a mut fmt::Formatter<'b>);

        impl io::Write for Adaptor<'_, '_> {
            fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
                let s = std::str::from_utf8(buf).map_err(|_| io::ErrorKind::Other)?;
                self.0.write_str(s).map_err(|_| io::ErrorKind::Other)?;
                Ok(buf.len())
            }

            fn flush(&mut self) -> io::Result<()> {
                Ok(())
            }
        }

        let mut cache = self.0.source_cache.borrow_mut();
        let color = !f.alternate();
        for error in &self.0.errors {
            error
                .report(color)
                .write(&mut *cache, Adaptor(f))
                .map_err(|_| fmt::Error)?
        }
        Ok(())
    }
}

impl ariadne::Cache<FileId> for SourceCache {
    fn fetch(&mut self, file_id: &FileId) -> Result<&ariadne::Source, Box<dyn fmt::Debug + '_>> {
        struct NotFound;
        impl fmt::Debug for NotFound {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("source file not found")
            }
        }
        match self.cache.entry(*file_id) {
            Entry::Occupied(entry) => Ok(entry.into_mut()),
            Entry::Vacant(entry) => match self.sources.get(file_id) {
                Some(file) => Ok(entry.insert(ariadne::Source::from(file.source_text()))),
                None => Err(Box::new(NotFound)),
            },
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
            let source_file = self.sources.get(file_id)?;
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

impl fmt::Debug for Diagnostics {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

impl ariadne::Span for NodeLocation {
    type SourceId = FileId;

    fn source(&self) -> &FileId {
        &self.file_id
    }

    fn start(&self) -> usize {
        self.offset()
    }

    fn end(&self) -> usize {
        self.end_offset()
    }
}

/// Track used names in a recursive function.
struct RecursionStack {
    seen: IndexSet<NodeStr>,
}

impl RecursionStack {
    fn with_root(root: NodeStr) -> Self {
        let mut seen = IndexSet::new();
        seen.insert(root);
        Self { seen }
    }

    /// Return the actual API for tracking recursive uses.
    pub fn guard(&mut self) -> RecursionGuard<'_> {
        RecursionGuard(&mut self.seen)
    }
}

/// Track used names in a recursive function.
///
/// Pass the result of `guard.push(name)` to recursive calls. Use `guard.contains(name)` to check
/// if the name was used somewhere up the call stack. When a guard is dropped, its name is removed
/// from the list.
struct RecursionGuard<'a>(&'a mut IndexSet<NodeStr>);
impl RecursionGuard<'_> {
    /// Mark that we saw a name.
    fn push(&mut self, name: &NodeStr) -> RecursionGuard<'_> {
        debug_assert!(
            self.0.insert(name.clone()),
            "cannot push the same name twice to RecursionGuard, check contains() first"
        );
        RecursionGuard(self.0)
    }
    /// Check if we saw a name somewhere up the call stack.
    fn contains(&self, name: &NodeStr) -> bool {
        self.0.iter().any(|seen| seen == name)
    }
    /// Return the name where we started.
    fn first(&self) -> Option<&NodeStr> {
        self.0.first()
    }
}

impl Drop for RecursionGuard<'_> {
    fn drop(&mut self) {
        // This may already be empty if it's the original `stack.guard()` result, but that's fine
        self.0.pop();
    }
}
