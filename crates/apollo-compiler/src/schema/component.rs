use crate::Node;
use crate::NodeLocation;
use crate::NodeStr;
use std::hash;
use std::ops::Deref;
use std::ops::DerefMut;
use triomphe::Arc;

/// A component of a type or `schema`, for example a field of an object type.
///
/// Wraps a [`Node<T>`] and adds its origin: either a (schema or type) definition
/// or a specific extension.
///
/// Implements [`Deref`] and [`DerefMut`]
/// so that methods and fields of `Node<T>` and `T` can be accessed directly.
#[derive(Debug, Clone)]
pub struct Component<T> {
    pub origin: ComponentOrigin,
    pub node: Node<T>,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum ComponentOrigin {
    Definition,
    Extension(ExtensionId),
}

/// Represents the identity of a schema extension or type extension.
///
/// Compares equal to its clones but not to other `ExtensionId`s created separately,
/// even if they contain the same source location.
#[derive(Debug, Clone, Eq)]
pub struct ExtensionId {
    arc: Arc<Option<NodeLocation>>,
}

impl ExtensionId {
    pub fn new<T>(extension: &Node<T>) -> Self {
        Self {
            arc: Arc::new(extension.location()),
        }
    }

    pub fn location(&self) -> Option<NodeLocation> {
        *self.arc
    }

    pub fn same_location<T>(&self, node: T) -> Node<T> {
        Node::new_opt_location(node, self.location())
    }
}

impl PartialEq for ExtensionId {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.arc, &other.arc)
    }
}

impl hash::Hash for ExtensionId {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        Arc::as_ptr(&self.arc).hash(state);
    }
}

impl ComponentOrigin {
    pub fn extension_id(&self) -> Option<&ExtensionId> {
        match self {
            ComponentOrigin::Definition => None,
            ComponentOrigin::Extension(id) => Some(id),
        }
    }
}

impl<T> Component<T> {
    /// Mark `node` as coming from a synthetic (no source location) definition (not an extension)
    pub fn new(node: T) -> Self {
        Self {
            origin: ComponentOrigin::Definition,
            node: Node::new(node),
        }
    }
}

impl<T: hash::Hash> hash::Hash for Component<T> {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.node.hash(state); // ignore `origin`
    }
}

impl<T: Eq> Eq for Component<T> {}

impl<T: PartialEq> PartialEq for Component<T> {
    fn eq(&self, other: &Self) -> bool {
        self.node == other.node // ignore `origin`
    }
}

impl<T> Deref for Component<T> {
    type Target = Node<T>;

    fn deref(&self) -> &Self::Target {
        &self.node
    }
}

impl<T> DerefMut for Component<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.node
    }
}

impl<T> AsRef<T> for Component<T> {
    fn as_ref(&self) -> &T {
        &self.node
    }
}

impl<T> From<T> for Component<T> {
    fn from(node: T) -> Self {
        Component::new(node)
    }
}

/// A string component of a type or `schema`, for example the name of a union member type.
///
/// Wraps a [`NodeStr`] and adds its origin: either a (`schema` or type) definition
/// or a specific extension.
///
/// Implements [`Deref`]
/// so that methods and fields of `NodeStr` and [`str`] can be accessed directly.
#[derive(Debug, Clone)]
pub struct ComponentStr {
    pub origin: ComponentOrigin,
    pub node: NodeStr,
}

impl ComponentStr {
    /// Mark `value` as coming from a synthetic (no source location) definition (not an extension)
    #[inline]
    pub fn new(value: &str) -> Self {
        Self {
            origin: ComponentOrigin::Definition,
            node: NodeStr::new(value),
        }
    }
}

impl hash::Hash for ComponentStr {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.node.hash(state); // ignore `origin`
    }
}

impl Eq for ComponentStr {}

impl PartialEq for ComponentStr {
    fn eq(&self, other: &Self) -> bool {
        self.node == other.node // ignore `origin`
    }
}

impl PartialEq<str> for ComponentStr {
    fn eq(&self, other: &str) -> bool {
        self.as_str() == other
    }
}

impl Deref for ComponentStr {
    type Target = NodeStr;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.node
    }
}

impl std::borrow::Borrow<NodeStr> for ComponentStr {
    fn borrow(&self) -> &NodeStr {
        self
    }
}

impl std::borrow::Borrow<str> for ComponentStr {
    fn borrow(&self) -> &str {
        self
    }
}

impl AsRef<str> for ComponentStr {
    fn as_ref(&self) -> &str {
        self
    }
}

impl From<&'_ str> for ComponentStr {
    fn from(value: &'_ str) -> Self {
        Self::new(value)
    }
}

impl From<&'_ String> for ComponentStr {
    fn from(value: &'_ String) -> Self {
        Self::new(value)
    }
}

impl From<String> for ComponentStr {
    fn from(value: String) -> Self {
        Self::new(&value)
    }
}
