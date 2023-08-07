use rowan::TextRange;
use std::collections::hash_map::RandomState;
use std::hash::BuildHasher;
use std::hash::Hash;
use std::hash::Hasher;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;
use std::sync::OnceLock;
use triomphe::Arc;

/// A shared reference to some MIR node.
///
/// Similar to [`std::sync::Arc`] but:
///
/// * Contains an optional [`TextRange`], indicating the location of the node within a parsed input file.
/// * Implements [`PartialEq`] and [`Hash`] based on the `T` node, excluding the source location:
///   nodes at different locations with the same content compare equal.
/// * Caches the result of hashing.
///   As a consequence, using `<Ref<T> as std::hash::Hash>` produces the hash of a hash,
///   different from `<T as std::hash::Hash>`.
/// * Does not support weak references
#[derive(Debug, Clone)]
pub struct Ref<T>(Arc<RefInner<T>>);

#[derive(Debug)]
struct RefInner<T> {
    /// `u32::MAX..u32::MAX`: unknown
    location: TextRange,
    /// Zero: not computed yet
    cached_hash: AtomicU64,
    node: T,
}

const CACHED_HASH_NOT_COMPUTED_YET: u64 = 0;

const _: () = {
    assert!(std::mem::size_of::<RefInner<()>>() == 16);
};

impl<T: Clone> Clone for RefInner<T> {
    fn clone(&self) -> Self {
        Self {
            location: self.location.clone(),
            cached_hash: AtomicU64::new(self.cached_hash.load(Ordering::Relaxed)),
            node: self.node.clone(),
        }
    }
}

impl<T> Ref<T> {
    pub fn new(node: T) -> Self {
        Self(Arc::new(RefInner {
            location: TextRange::new(u32::MAX.into(), u32::MAX.into()),
            cached_hash: AtomicU64::new(CACHED_HASH_NOT_COMPUTED_YET),
            node,
        }))
    }

    pub fn with_location(node: T, location: TextRange) -> Self {
        Self(Arc::new(RefInner {
            location,
            cached_hash: AtomicU64::new(CACHED_HASH_NOT_COMPUTED_YET),
            node,
        }))
    }

    pub fn location(&self) -> Option<&TextRange> {
        if u32::from(self.0.location.start()) != u32::MAX {
            Some(&self.0.location)
        } else {
            None
        }
    }

    pub fn make_mut(&mut self) -> &mut T
    where
        T: Clone,
    {
        let inner = Arc::make_mut(&mut self.0);
        // Clear the cache as mutation through the returned `&mut T` would invalidate it
        *inner.cached_hash.get_mut() = 0;
        &mut inner.node
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

impl<T: Hash> Hash for Ref<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash_of_node().hash(state);
    }
}

impl<T: Hash> RefInner<T> {
    fn hash_of_node(&self) -> u64 {
        let hash = self.cached_hash.load(Ordering::Relaxed);
        if hash != CACHED_HASH_NOT_COMPUTED_YET {
            hash
        } else {
            #[cold]
            #[inline(never)]
            fn slow_path<T: Hash>(inner: &RefInner<T>) -> u64 {
                static SHARED_RANDOM: OnceLock<RandomState> = OnceLock::new();
                let mut hasher = SHARED_RANDOM.get_or_init(RandomState::new).build_hasher();
                inner.node.hash(&mut hasher);
                let mut hash = hasher.finish();
                // Don’t use the marker value for an actual hash
                if hash == CACHED_HASH_NOT_COMPUTED_YET {
                    hash += 1
                }
                inner.cached_hash.store(hash, Ordering::Relaxed);
                hash
            }
            slow_path(self)
        }
    }
}
