use crate::hir::HirNodeLocation;
use crate::FileId;
use apollo_parser::mir::Harc;
use apollo_parser::mir::Ranged;
use triomphe::Arc;

/// Wraps a `Harc<Ranged<T>>` typically from a MIR document and adds an optional `FileId`.
#[derive(Debug, Hash, PartialEq, Eq)]
pub struct Located<T> {
    file_id: Option<FileId>,
    harc: Harc<Ranged<T>>,
}

impl<T> Clone for Located<T> {
    fn clone(&self) -> Self {
        Self {
            file_id: self.file_id.clone(),
            harc: self.harc.clone(),
        }
    }
}

#[derive(Debug, Hash, PartialEq, Eq)]
pub struct LocatedBorrow<'a, T> {
    file_id: Option<FileId>,
    harc: &'a Harc<Ranged<T>>,
}

impl<'a, T> Copy for LocatedBorrow<'a, T> {}

impl<'a, T> Clone for LocatedBorrow<'a, T> {
    fn clone(&self) -> Self {
        Self {
            file_id: self.file_id.clone(),
            harc: self.harc,
        }
    }
}

impl<T> std::ops::Deref for Located<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.harc.node
    }
}

impl<T> std::ops::Deref for LocatedBorrow<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.harc.node
    }
}

impl<T> Located<T> {
    pub fn no_location(node: T) -> Self {
        Self {
            file_id: None,
            harc: Harc::new(Ranged::no_location(node)),
        }
    }

    pub fn no_file_id(harc: Harc<Ranged<T>>) -> Self {
        Self {
            file_id: None,
            harc,
        }
    }

    pub fn with_file_id(value: Harc<Ranged<T>>, file_id: FileId) -> Self {
        Self {
            file_id: Some(file_id),
            harc: value,
        }
    }

    pub fn same_file_id<'a, C>(&self, harc: &'a Harc<Ranged<C>>) -> LocatedBorrow<'a, C> {
        LocatedBorrow {
            file_id: self.file_id,
            harc,
        }
    }

    pub fn borrow(&self) -> LocatedBorrow<'_, T> {
        self.same_file_id(&self.harc)
    }

    /// Return a mutable reference, with copy-on-write semantics:
    ///
    /// Clone the value behind `Arc` if other `Arc`s point to the same allocation.
    pub fn make_mut(&mut self) -> &mut T
    where
        T: Clone,
    {
        self.harc.make_mut()
    }

    pub fn source_location(&self) -> Option<HirNodeLocation> {
        self.borrow().source_location()
    }
}

impl<'a, T> LocatedBorrow<'a, T> {
    pub fn with_file_id(value: &'a Harc<Ranged<T>>, file_id: FileId) -> Self {
        Self {
            file_id: Some(file_id),
            harc: value,
        }
    }

    pub fn source_location(&self) -> Option<HirNodeLocation> {
        Some(HirNodeLocation {
            file_id: self.file_id?,
            text_range: *self.harc.location()?,
        })
    }

    pub fn to_owned(&self) -> Located<T> {
        Located {
            file_id: self.file_id,
            harc: Harc::clone(self.harc),
        }
    }

    /// Returns the given `node` at the same location as `self` (e.g. for a type conversion).
    pub fn same_location<U>(&self, node: U) -> Located<U> {
        Located {
            file_id: self.file_id,
            harc: Harc::new(self.harc.at_same_location(node)),
        }
    }

    pub fn component<U>(
        &self,
        harc: &Harc<Ranged<U>>,
        extension: Option<&ExtensionId>,
    ) -> Component<U> {
        Component {
            extension: extension.cloned(),
            component: Located {
                file_id: self.file_id,
                harc: harc.clone(),
            },
        }
    }
}

/// Represents the identity of a schema extension or type extension.
///
/// Compares equal to its clones, but not to other `ExtensionId`s created separately
/// even if they contain the same source location.
#[derive(Debug, Clone, Eq)]
pub struct ExtensionId {
    extension_location: Arc<Option<HirNodeLocation>>,
}

impl ExtensionId {
    pub fn new<T>(extension: LocatedBorrow<T>) -> Self {
        Self {
            extension_location: Arc::new(extension.source_location()),
        }
    }

    pub fn source_location(&self) -> Option<&HirNodeLocation> {
        (*self.extension_location).as_ref()
    }
}

impl PartialEq for ExtensionId {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.extension_location, &other.extension_location)
    }
}

impl std::hash::Hash for ExtensionId {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        Arc::as_ptr(&self.extension_location).hash(state);
    }
}

/// Wraps a `Located<T>` and additionally keeps track of whether a component
/// comes from a schema extension or type extension, and which one.
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Component<T> {
    pub extension: Option<ExtensionId>,
    component: Located<T>,
}

impl<T> Component<T> {
    /// Mark `node` is coming from a synthetic (no source location) definition (not an extension)
    pub fn no_location(node: T) -> Self {
        Self {
            extension: None,
            component: Located::no_location(node),
        }
    }

    /// Returns `None` for components from the main definition,
    /// or `Some` for those from a schema extension or type extension.
    pub fn extension_id(&self) -> Option<&ExtensionId> {
        self.extension.as_ref()
    }

    pub fn source_location(&self) -> Option<HirNodeLocation> {
        self.component.source_location()
    }
}

impl<T> std::ops::Deref for Component<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &*self.component
    }
}
