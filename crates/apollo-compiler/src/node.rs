use crate::parser::FileId;
use crate::parser::LineColumn;
use crate::parser::SourceMap;
use crate::parser::SourceSpan;
use crate::schema::Component;
use crate::schema::ComponentOrigin;
use std::fmt;
use std::hash::Hash;
use std::hash::Hasher;
use std::ops::Range;
use triomphe::HeaderSlice;

/// A thread-safe reference-counted smart pointer for GraphQL nodes.
///
/// Similar to [`std::sync::Arc<T>`] but:
///
/// * In addition to `T`, contains an optional [`SourceSpan`].
///   This location notably allows diagnostics to point to relevant parts of parsed input files.
/// * Weak references are not supported.
#[derive(serde::Deserialize)]
#[serde(from = "T")]
pub struct Node<T: ?Sized>(triomphe::Arc<HeaderSlice<Header, T>>);

#[derive(Clone)]
struct Header {
    location: Option<SourceSpan>,
}

impl<T> Node<T> {
    /// Create a new `Node` for something parsed from the given source location
    #[inline]
    pub fn new_parsed(node: T, location: SourceSpan) -> Self {
        Self::new_opt_location(node, Some(location))
    }

    /// Create a new `Node` for something created programatically, not parsed from a source file
    pub fn new(node: T) -> Self {
        Self::new_opt_location(node, None)
    }

    pub(crate) fn new_opt_location(node: T, location: Option<SourceSpan>) -> Self {
        Self(triomphe::Arc::new(HeaderSlice {
            header: Header { location },
            slice: node,
        }))
    }
}

impl Node<str> {
    /// Create a new `Node<str>` for a string parsed from the given source location
    #[inline]
    pub fn new_str_parsed(node: &str, location: SourceSpan) -> Self {
        Self::new_str_opt_location(node, Some(location))
    }

    /// Create a new `Node<str>` for a string created programatically, not parsed from a source file
    pub fn new_str(node: &str) -> Self {
        Self::new_str_opt_location(node, None)
    }

    pub(crate) fn new_str_opt_location(node: &str, location: Option<SourceSpan>) -> Self {
        Self(triomphe::Arc::from_header_and_str(
            Header { location },
            node,
        ))
    }

    pub fn as_str(&self) -> &str {
        self
    }
}

impl<T: ?Sized> Node<T> {
    /// If this node was parsed from a source file, returns the file ID and source span
    /// (start and end byte offsets) within that file.
    pub fn location(&self) -> Option<SourceSpan> {
        self.0.header.location
    }

    /// Whether this node is located in `FileId::BUILT_IN`,
    /// which defines built-in directives, built-in scalars, and introspection types.
    pub fn is_built_in(&self) -> bool {
        self.location().map(|l| l.file_id()) == Some(FileId::BUILT_IN)
    }

    /// If this node contains a location, convert it to the line and column numbers.
    pub fn line_column_range(&self, sources: &SourceMap) -> Option<Range<LineColumn>> {
        self.location()?.line_column_range(sources)
    }

    /// Returns the given `node` at the same location as `self` (e.g. for a type conversion).
    pub fn same_location<U>(&self, node: U) -> Node<U> {
        Node::new_opt_location(node, self.0.header.location)
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
        &mut inner.slice
    }

    /// Returns a mutable reference to `T` if this `Node` is uniquely owned
    pub fn get_mut(&mut self) -> Option<&mut T> {
        triomphe::Arc::get_mut(&mut self.0).map(|inner| &mut inner.slice)
    }
}

impl<T: ?Sized> std::ops::Deref for Node<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0.slice
    }
}

impl<T: ?Sized> Clone for Node<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T: Default> Default for Node<T> {
    fn default() -> Self {
        Self::new(T::default())
    }
}

impl<T: ?Sized + fmt::Debug> fmt::Debug for Node<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(location) = self.location() {
            write!(f, "{location:?} ")?
        }
        self.0.slice.fmt(f)
    }
}

impl<T: ?Sized + fmt::Display> fmt::Display for Node<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        T::fmt(self, f)
    }
}

impl<T: ?Sized + Eq> Eq for Node<T> {}

impl<T: ?Sized + PartialEq> PartialEq for Node<T> {
    fn eq(&self, other: &Self) -> bool {
        self.ptr_eq(other) // fast path
        || self.0.slice == other.0.slice // location not included
    }
}

impl<T: ?Sized + Hash> Hash for Node<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.slice.hash(state)
    }
}

impl<T: ?Sized> std::borrow::Borrow<T> for Node<T> {
    fn borrow(&self) -> &T {
        self
    }
}

impl<T: ?Sized> AsRef<T> for Node<T> {
    fn as_ref(&self) -> &T {
        self
    }
}

impl<T> From<T> for Node<T> {
    fn from(node: T) -> Self {
        Self::new(node)
    }
}

impl From<&'_ str> for Node<str> {
    fn from(node: &'_ str) -> Self {
        Self::new_str(node)
    }
}

impl From<&'_ String> for Node<str> {
    fn from(node: &'_ String) -> Self {
        Self::new_str(node)
    }
}

impl From<String> for Node<str> {
    fn from(node: String) -> Self {
        Self::new_str(&node)
    }
}

impl From<&'_ Node<str>> for String {
    fn from(node: &'_ Node<str>) -> Self {
        node.as_str().to_owned()
    }
}

impl From<Node<str>> for String {
    fn from(node: Node<str>) -> Self {
        node.as_str().to_owned()
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
