pub use crate::hir::HirNodeLocation as NodeLocation;
use crate::schema::Component;
use crate::schema::ComponentOrigin;
use crate::Arc;
use crate::FileId;
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

impl<T> Node<T> {
    /// Create a new `Node` for something parsed from the given source location
    pub fn new_parsed(node: T, location: NodeLocation) -> Self {
        Self(Arc::new(NodeInner {
            location: Some(location),
            node,
        }))
    }

    /// Create a new `Node` for something created programatically, not parsed from a source file
    pub fn new_synthetic(node: T) -> Self {
        Self(Arc::new(NodeInner {
            location: None,
            node,
        }))
    }

    pub fn location(&self) -> Option<&NodeLocation> {
        self.0.location.as_ref()
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
            write!(
                f,
                "@{:?}:{}..{} ",
                location.file_id().to_i64(),
                location.offset(),
                location.end_offset()
            )?
        }
        self.0.node.fmt(f)
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
