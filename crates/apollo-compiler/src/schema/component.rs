use crate::parser::SourceSpan;
use crate::Name;
use crate::Node;
use std::fmt;
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
pub struct Component<T: ?Sized> {
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
    arc: Arc<Option<SourceSpan>>,
}

impl ExtensionId {
    pub fn new<T>(extension: &Node<T>) -> Self {
        Self {
            arc: Arc::new(extension.location()),
        }
    }

    pub fn location(&self) -> Option<SourceSpan> {
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

impl<T: ?Sized + hash::Hash> hash::Hash for Component<T> {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.node.hash(state); // ignore `origin`
    }
}

impl<T: ?Sized + Eq> Eq for Component<T> {}

impl<T: ?Sized + PartialEq> PartialEq for Component<T> {
    fn eq(&self, other: &Self) -> bool {
        self.node == other.node // ignore `origin`
    }
}

impl<T: ?Sized> Deref for Component<T> {
    type Target = Node<T>;

    fn deref(&self) -> &Self::Target {
        &self.node
    }
}

impl<T: ?Sized> DerefMut for Component<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.node
    }
}

impl<T: ?Sized> AsRef<T> for Component<T> {
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
/// Wraps a [`Name`] and adds its origin: either a (`schema` or type) definition
/// or a specific extension.
///
/// Implements [`Deref`]
/// so that methods and fields of `Name` and [`str`] can be accessed directly.
#[derive(Debug, Clone)]
pub struct ComponentName {
    pub origin: ComponentOrigin,
    pub name: Name,
}

impl From<&Name> for ComponentName {
    fn from(value: &Name) -> Self {
        value.to_component(ComponentOrigin::Definition)
    }
}

impl From<Name> for ComponentName {
    fn from(value: Name) -> Self {
        value.to_component(ComponentOrigin::Definition)
    }
}

impl hash::Hash for ComponentName {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state); // ignore `origin`
    }
}

impl Eq for ComponentName {}

impl PartialEq for ComponentName {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name // ignore `origin`
    }
}

impl PartialEq<str> for ComponentName {
    fn eq(&self, other: &str) -> bool {
        self.as_str() == other
    }
}
impl PartialEq<Name> for ComponentName {
    fn eq(&self, other: &Name) -> bool {
        self.name == *other
    }
}

impl Deref for ComponentName {
    type Target = Name;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.name
    }
}

impl std::borrow::Borrow<Name> for ComponentName {
    fn borrow(&self) -> &Name {
        self
    }
}

impl std::borrow::Borrow<str> for ComponentName {
    fn borrow(&self) -> &str {
        self
    }
}

impl AsRef<str> for ComponentName {
    fn as_ref(&self) -> &str {
        self
    }
}

impl fmt::Display for ComponentName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.name.fmt(f)
    }
}
