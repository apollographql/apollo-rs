use std::sync::Arc;
use std::{fmt, num::NonZeroI64, path::PathBuf, sync::atomic};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SourceType {
    Schema,
    Executable,
    Document,
    TextOnly,
}

/// Represents a GraphQL source file.
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Source {
    pub(crate) ty: SourceType,
    pub(crate) filename: PathBuf,
    pub(crate) text: Arc<String>,
    pub(crate) ast: Option<Arc<crate::ast::Document>>,
}

impl Source {
    pub fn source_type(&self) -> SourceType {
        self.ty
    }

    pub fn text(&self) -> &Arc<String> {
        &self.text
    }
}

impl From<&'_ Arc<crate::SourceFile>> for Source {
    fn from(file: &'_ Arc<crate::SourceFile>) -> Self {
        Self {
            ty: crate::database::SourceType::TextOnly,
            filename: file.path.clone(),
            text: Arc::new(file.source_text.clone()),
            ast: None,
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct FileId {
    id: NonZeroI64,
}

impl fmt::Debug for FileId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.id.fmt(f)
    }
}

/// The next file ID to use. This is global so file IDs do not conflict between different compiler
/// instances.
static NEXT: atomic::AtomicI64 = atomic::AtomicI64::new(INITIAL);
static INITIAL: i64 = 1;

impl FileId {
    /// The ID of the file implicitly added to type systems, for built-in scalars and introspection types
    pub const BUILT_IN: Self = Self::const_new(-1);

    /// Passed to Ariadne to create a report without a location
    pub(crate) const NONE: Self = Self::const_new(-2);

    pub(crate) const HACK_TMP: Self = Self::const_new(-3);
    pub(crate) const HACK_TMP_2: Self = Self::const_new(-4);

    // Returning a different value every time does not sound like good `impl Default`
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let id = NEXT.fetch_add(1, atomic::Ordering::AcqRel);
        Self {
            id: NonZeroI64::new(id).unwrap(),
        }
    }

    /// Reset file ID back to 1, used to get consistent results in tests.
    ///
    /// All tests in the process must use `#[serial_test::serial]`
    #[doc(hidden)]
    pub fn reset() {
        NEXT.store(INITIAL, atomic::Ordering::Release)
    }

    const fn const_new(id: i64) -> Self {
        // TODO: use unwrap() when const-stable https://github.com/rust-lang/rust/issues/67441
        if let Some(id) = NonZeroI64::new(id) {
            Self { id }
        } else {
            panic!()
        }
    }
}
