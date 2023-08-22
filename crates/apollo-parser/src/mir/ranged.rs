use rowan::TextRange;
use std::hash::Hash;

/// Wrap a MIR node, adding an optional [`TextRange`] location within a parsed source file.
///
/// Implements [`PartialEq`] and [`Hash`] based on the `T` node, excluding the source location:
/// nodes at different locations with the same content compare equal.
#[derive(Debug, Clone, Copy)]
pub struct Ranged<T> {
    /// `u32::MAX..u32::MAX`: unknown
    location: TextRange,
    pub node: T,
}

impl<T> Ranged<T> {
    pub fn no_location(node: T) -> Self {
        Self {
            location: TextRange::new(u32::MAX.into(), u32::MAX.into()),
            node,
        }
    }

    pub fn with_location(node: T, location: TextRange) -> Self {
        debug_assert!(u32::from(location.start()) != u32::MAX);
        Self { location, node }
    }

    pub fn location(&self) -> Option<&TextRange> {
        if u32::from(self.location.start()) != u32::MAX {
            Some(&self.location)
        } else {
            None
        }
    }

    /// Returns the given `node` at the same location as `self` (e.g. for a type conversion).
    pub fn at_same_location<U>(&self, node: U) -> Ranged<U> {
        Ranged {
            location: self.location,
            node,
        }
    }
}

impl<T> std::ops::Deref for Ranged<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.node
    }
}

impl<T> std::ops::DerefMut for Ranged<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.node
    }
}

impl<T: Eq> Eq for Ranged<T> {}

impl<T: PartialEq> PartialEq for Ranged<T> {
    fn eq(&self, other: &Self) -> bool {
        self.node == other.node // location not included
    }
}

impl<T: Hash> Hash for Ranged<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.node.hash(state); // location not included
    }
}
