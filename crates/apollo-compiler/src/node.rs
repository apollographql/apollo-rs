use crate::execution::GraphQLLocation;
use crate::schema::Component;
use crate::schema::ComponentOrigin;
use crate::SourceMap;
use apollo_parser::SyntaxNode;
use rowan::TextRange;
use std::collections::hash_map::RandomState;
use std::fmt;
use std::hash::BuildHasher;
use std::hash::Hash;
use std::hash::Hasher;
use std::num::NonZeroU64;
use std::sync::atomic;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;
use std::sync::OnceLock;
use triomphe::HeaderSlice;

/// A thread-safe reference-counted smart pointer for GraphQL nodes.
///
/// Similar to [`std::sync::Arc<T>`] but:
///
/// * In addition to `T`, contains an optional [`NodeLocation`].
///   This location notably allows diagnostics to point to relevant parts of parsed input files.
/// * [`std::hash::Hash`] is implemented by caching the result of hashing `T`.
/// * Weak references are not supported.
///
/// For the cache to be correct, **`T` is expected to have a stable hash**
/// a long as no `&mut T` exclusive reference to it is given out.
/// Generally this excludes interior mutability.
///
/// `Node<T>` cannot implement [`Borrow<T>`][std::borrow::Borrow] because `Node<T> as Hash`
/// produces a result (the hash of the cached hash) different from `T as Hash`.
#[derive(serde::Deserialize)]
#[serde(from = "T")]
pub struct Node<T: ?Sized>(triomphe::Arc<HeaderSlice<Header, T>>);

struct Header {
    location: Option<NodeLocation>,
    hash_cache: AtomicU64,
}

const HASH_NOT_COMPUTED_YET: u64 = 0;

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
    id: NonZeroU64,
}

#[derive(Copy, Clone)]
pub(crate) struct TaggedFileId {
    tag_and_id: NonZeroU64,
}

impl<T> Node<T> {
    /// Create a new `Node` for something parsed from the given source location
    #[inline]
    pub fn new_parsed(node: T, location: NodeLocation) -> Self {
        Self::new_opt_location(node, Some(location))
    }

    /// Create a new `Node` for something created programatically, not parsed from a source file
    pub fn new(node: T) -> Self {
        Self::new_opt_location(node, None)
    }

    pub(crate) fn new_opt_location(node: T, location: Option<NodeLocation>) -> Self {
        Self(triomphe::Arc::new(HeaderSlice {
            header: Header {
                location,
                hash_cache: AtomicU64::new(HASH_NOT_COMPUTED_YET),
            },
            slice: node,
        }))
    }
}

impl Node<str> {
    /// Create a new `Node<str>` for a string parsed from the given source location
    #[inline]
    pub fn new_str_parsed(node: &str, location: NodeLocation) -> Self {
        Self::new_str_opt_location(node, Some(location))
    }

    /// Create a new `Node<str>` for a string created programatically, not parsed from a source file
    pub fn new_str(node: &str) -> Self {
        Self::new_str_opt_location(node, None)
    }

    pub(crate) fn new_str_opt_location(node: &str, location: Option<NodeLocation>) -> Self {
        Self(triomphe::Arc::from_header_and_str(
            Header {
                location,
                hash_cache: AtomicU64::new(HASH_NOT_COMPUTED_YET),
            },
            node,
        ))
    }

    pub fn as_str(&self) -> &str {
        self
    }
}

impl<T: ?Sized> Node<T> {
    pub fn location(&self) -> Option<NodeLocation> {
        self.0.header.location
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
        // Clear the cache as mutation through the returned `&mut T` may invalidate it
        *inner.header.hash_cache.get_mut() = HASH_NOT_COMPUTED_YET;
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
        || self.0.slice == other.0.slice // location and hash_cache not included
    }
}

impl<T: ?Sized + Hash> Hash for Node<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let hash = self.0.header.hash_cache.load(Ordering::Relaxed);
        if hash != HASH_NOT_COMPUTED_YET {
            // cache hit
            hash
        } else {
            hash_slow_path(&self.0)
        }
        .hash(state)
    }
}

// It is possible for multiple threads to race and take this path for the same `NodeInner`.
// This is ok as they should compute the same result.
// We save on the extra space that `OnceLock<u64>` would occupy,
// at the cost of extra computation in the unlikely case of this race.
#[cold]
#[inline(never)]
fn hash_slow_path<T: ?Sized + Hash>(inner: &HeaderSlice<Header, T>) -> u64 {
    /// We share a single `BuildHasher` process-wide,
    /// not only for the race described above but also
    /// so that multiple `HarcInner`’s with the same contents have the same hash.
    static SHARED_RANDOM: OnceLock<RandomState> = OnceLock::new();
    let mut hash = SHARED_RANDOM
        .get_or_init(RandomState::new)
        .hash_one(&inner.slice);
    // Don’t use the marker value for an actual hash
    if hash == HASH_NOT_COMPUTED_YET {
        hash += 1
    }
    inner.header.hash_cache.store(hash, Ordering::Relaxed);
    hash
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

impl Clone for Header {
    fn clone(&self) -> Self {
        Self {
            location: self.location,
            hash_cache: AtomicU64::new(self.hash_cache.load(Ordering::Relaxed)),
        }
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
        let tag_and_id = if tag {
            debug_assert!((id.id.get() & TAG) == 0);
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
