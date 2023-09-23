use crate::schema::Component;
use crate::schema::ComponentOrigin;
use crate::Arc;
use crate::FileId;
use apollo_parser::SyntaxNode;
use rowan::TextRange;
use std::fmt;
use std::hash;

/// Smart pointer to some GraphQL node.
///
/// Similar to [`apollo_compiler::Arc`][Arc] (thread-safe, reference-counted, cached `Hash`)
/// but additionally carries an optional [`NodeLocation`].
/// This location notably allows diagnostics to point to relevant parts of parsed input files.
///
/// Like `Arc`, `Node<T>` cannot implement [`Borrow<T>`][std::borrow::Borrow]
/// because `Node<T> as Hash` produces a result (the hash of the cached hash)
/// different from `T as Hash`.
#[derive(Hash, Eq, PartialEq)]
pub struct Node<T>(Arc<NodeInner<T>>);

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

impl<T> Node<T> {
    /// Create a new `Node` for something parsed from the given source location
    pub fn new_parsed(node: T, location: NodeLocation) -> Self {
        Self(Arc::new(NodeInner {
            location: Some(location),
            node,
        }))
    }

    /// Create a new `Node` for something created programatically, not parsed from a source file
    pub fn new(node: T) -> Self {
        Self(Arc::new(NodeInner {
            location: None,
            node,
        }))
    }

    pub fn location(&self) -> Option<NodeLocation> {
        self.0.location
    }

    /// Whether this node is located in `FileId::BUILT_IN`,
    /// which defines built-in directives, built-in scalars, and introspection types.
    pub fn is_built_in(&self) -> bool {
        self.location().map(|l| l.file_id()) == Some(FileId::BUILT_IN)
    }

    /// Returns the given `node` at the same location as `self` (e.g. for a type conversion).
    pub fn same_location<U>(&self, node: U) -> Node<U> {
        Node(Arc::new(NodeInner {
            location: self.0.location,
            node,
        }))
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
        self.0.ptr_eq(&other.0)
    }

    /// Returns a mutable reference to `T`, cloning it if necessary
    ///
    /// See [`Arc::make_mut`].
    pub fn make_mut(&mut self) -> &mut T
    where
        T: Clone,
    {
        let inner = self.0.make_mut();
        // TODO: should the `inner.location` be set to `None` here?
        // After a node is mutated it is kind of not from that source location anymore
        &mut inner.node
    }

    /// Returns a mutable reference to `T` if this `Node` is uniquely owned
    pub fn get_mut(&mut self) -> Option<&mut T> {
        self.0.get_mut().map(|inner| &mut inner.node)
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

impl<T: Eq> Eq for NodeInner<T> {}

impl<T: PartialEq> PartialEq for NodeInner<T> {
    fn eq(&self, other: &Self) -> bool {
        self.node == other.node // location not included
    }
}

impl<T: hash::Hash> hash::Hash for NodeInner<T> {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.node.hash(state) // location not included
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
