use crate::execution::GraphQLLocation;
use crate::schema::Component;
use crate::schema::ComponentOrigin;
use crate::SourceMap;
use apollo_parser::SyntaxNode;
use rowan::TextRange;
use std::fmt;
use std::hash::Hash;
use std::hash::Hasher;
use std::num::NonZeroI64;
use std::sync::atomic;

/// A thread-safe reference-counted smart pointer for GraphQL nodes.
///
/// Similar to [`std::sync::Arc<T>`] but:
///
/// * In addition to `T`, contains an optional [`NodeLocation`].
///   This location notably allows diagnostics to point to relevant parts of parsed input files.
/// * Weak references are not supported.
#[derive(serde::Deserialize)]
#[serde(from = "T")]
pub struct Node<T>(triomphe::Arc<NodeInner<T>>);

#[derive(Clone)]
struct NodeInner<T> {
    location: Option<NodeLocation>,
    node: T,
}

/// The source location of a parsed node: file ID and range within that file.
#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct NodeLocation {
    pub(crate) file_id: FileId,
    pub(crate) text_range: TextRange,
}

/// Integer identifier for a parsed source file.
///
/// Used internally to support validating for example a schema built from multiple source files,
/// and having diagnostics point to relevant sources.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct FileId {
    id: NonZeroI64,
}

impl<T> Node<T> {
    /// Create a new `Node` for something parsed from the given source location
    #[inline]
    pub fn new_parsed(node: T, location: NodeLocation) -> Self {
        Self::new_opt_location(node, Some(location))
    }

    /// Create a new `Node` for something created programatically, not parsed from a source file
    #[inline]
    pub fn new(node: T) -> Self {
        Self::new_opt_location(node, None)
    }

    pub(crate) fn new_opt_location(node: T, location: Option<NodeLocation>) -> Self {
        Self(triomphe::Arc::new(NodeInner { location, node }))
    }

    pub fn location(&self) -> Option<NodeLocation> {
        self.0.location
    }

    /// Whether this node is located in `FileId::BUILT_IN`,
    /// which defines built-in directives, built-in scalars, and introspection types.
    pub fn is_built_in(&self) -> bool {
        self.location().map(|l| l.file_id()) == Some(FileId::BUILT_IN)
    }

    /// If this node contains a location, convert it to line and column numbers
    pub fn line_column(&self, sources: &SourceMap) -> Option<GraphQLLocation> {
        GraphQLLocation::from_node(sources, self.location())
    }

    /// Returns the given `node` at the same location as `self` (e.g. for a type conversion).
    pub fn same_location<U>(&self, node: U) -> Node<U> {
        Node::new_opt_location(node, self.0.location)
    }

    pub fn to_component(&self, origin: ComponentOrigin) -> Component<T> {
        Component {
            origin,
            node: self.clone(),
        }
    }

    // `Arc` APIs

    /// Returns whether two `Node`s point to the same memory allocation
    pub fn ptr_eq(&self, other: &Self) -> bool {
        triomphe::Arc::ptr_eq(&self.0, &other.0)
    }

    /// Returns a mutable reference to `T`, cloning it if necessary
    ///
    /// This is functionally equivalent to [`Arc::make_mut`][mm] from the standard library.
    ///
    /// If this `Node` is uniquely owned, `make_mut()` will provide a mutable
    /// reference to the contents. If not, `make_mut()` will create a _new_ `Node`
    /// with a clone of the contents, update `self` to point to it, and provide
    /// a mutable reference to its contents.
    ///
    /// This is useful for implementing copy-on-write schemes where you wish to
    /// avoid copying things if your `Node` is not shared.
    ///
    /// [mm]: https://doc.rust-lang.org/stable/std/sync/struct.Arc.html#method.make_mut
    pub fn make_mut(&mut self) -> &mut T
    where
        T: Clone,
    {
        let inner = triomphe::Arc::make_mut(&mut self.0);
        // TODO: should the `inner.location` be set to `None` here?
        // After a node is mutated it is kind of not from that source location anymore
        &mut inner.node
    }

    /// Returns a mutable reference to `T` if this `Node` is uniquely owned
    pub fn get_mut(&mut self) -> Option<&mut T> {
        triomphe::Arc::get_mut(&mut self.0).map(|inner| &mut inner.node)
    }
}

impl<T> std::ops::Deref for Node<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0.node
    }
}

impl<T> Clone for Node<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T: Default> Default for Node<T> {
    fn default() -> Self {
        Self::new(T::default())
    }
}

impl<T: fmt::Debug> fmt::Debug for Node<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(location) = self.location() {
            write!(f, "{location:?} ")?
        }
        self.0.node.fmt(f)
    }
}

impl<T: fmt::Display> fmt::Display for Node<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        T::fmt(self, f)
    }
}

impl<T: Eq> Eq for Node<T> {}

impl<T: PartialEq> PartialEq for Node<T> {
    fn eq(&self, other: &Self) -> bool {
        self.ptr_eq(other) // fast path
        || self.0.node == other.0.node // location not included
    }
}

impl<T: Hash> Hash for Node<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.node.hash(state)
    }
}

impl<T> std::borrow::Borrow<T> for Node<T> {
    fn borrow(&self) -> &T {
        self
    }
}

impl<T> AsRef<T> for Node<T> {
    fn as_ref(&self) -> &T {
        self
    }
}

impl<T> From<T> for Node<T> {
    fn from(node: T) -> Self {
        Self::new(node)
    }
}

impl NodeLocation {
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
                Some(NodeLocation {
                    file_id: start.file_id,
                    text_range: TextRange::new(start.text_range.start(), end.text_range.end()),
                })
            }
        }
    }
}

impl fmt::Debug for NodeLocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}..{} @{:?}",
            self.offset(),
            self.end_offset(),
            self.file_id,
        )
    }
}

impl<T: serde::Serialize> serde::Serialize for Node<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        T::serialize(self, serializer)
    }
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
