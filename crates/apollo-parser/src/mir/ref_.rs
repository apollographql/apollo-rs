use rowan::TextRange;
use triomphe::Arc;

/// A shared reference to some MIR node.
///
/// Similar to [`std::sync::Arc`] but:
///
/// * Contains an optional `TextRange`, indicating the location of the node within a parsed input file.
/// * Does not support weak references
#[derive(Debug, Clone)]
pub struct Ref<T>(Arc<RefInner<T>>);

#[derive(Debug, Clone)]
struct RefInner<T> {
    location: Option<TextRange>,
    node: T,
}

impl<T> Ref<T> {
    pub fn new(node: T) -> Self {
        Self(Arc::new(RefInner {
            location: None,
            node,
        }))
    }

    pub fn with_location(node: T, location: TextRange) -> Self {
        Self(Arc::new(RefInner {
            location: Some(location),
            node,
        }))
    }

    pub fn location(&self) -> Option<&TextRange> {
        self.0.location.as_ref()
    }

    pub fn make_mut(&mut self) -> &mut T
    where
        T: Clone,
    {
        &mut Arc::make_mut(&mut self.0).node
    }

    pub fn ptr_eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl<T> std::ops::Deref for Ref<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0.node
    }
}

impl<T: PartialEq> PartialEq for Ref<T> {
    fn eq(&self, other: &Self) -> bool {
        self.ptr_eq(other) // fast path
        || self.0.node == other.0.node // location not included
    }
}
