use std::collections::hash_map::RandomState;
use std::hash::BuildHasher;
use std::hash::Hash;
use std::hash::Hasher;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;
use std::sync::OnceLock;
use triomphe::Arc;

/// `Arc` with cached `Hash`
///
/// Like [`std::sync::Arc`] this is a thread-safe reference-counting pointer.
/// It differs in removing support for weak references and adding a cache for [`Hash`].
///
/// For this cache to be correct, `T` is expected to have a stable hash a long as no
/// `&mut T` exclusive reference to it is given out.
/// Generally this excludes interior mutability.
#[derive(Debug)]
pub struct Harc<T>(Arc<HarcInner<T>>);

#[derive(Debug)]
struct HarcInner<T> {
    /// Zero: not computed yet
    cached_hash: AtomicU64,
    value: T,
}

const CACHED_HASH_NOT_COMPUTED_YET: u64 = 0;

const _: () = {
    assert!(std::mem::size_of::<HarcInner<super::Ranged<()>>>() == 16);
};

impl<T> Clone for Harc<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T: Clone> Clone for HarcInner<T> {
    fn clone(&self) -> Self {
        Self {
            cached_hash: AtomicU64::new(self.cached_hash.load(Ordering::Relaxed)),
            value: self.value.clone(),
        }
    }
}

impl<T> Harc<T> {
    pub fn new(value: T) -> Self {
        Self(Arc::new(HarcInner {
            cached_hash: AtomicU64::new(CACHED_HASH_NOT_COMPUTED_YET),
            value,
        }))
    }

    /// Returns a mutable reference to `T`, cloning it if necessary
    ///
    /// This is functionally equivalent to [`Arc::make_mut`][mm] from the standard library.
    ///
    /// If this `Harc` is uniquely owned, `make_mut()` will provide a mutable
    /// reference to the contents. If not, `make_mut()` will create a _new_ `Harc`
    /// with a copy of the contents, update `self` to point to it, and provide
    /// a mutable reference to its contents.
    ///
    /// This is useful for implementing copy-on-write schemes where you wish to
    /// avoid copying things if your `Harc` is not shared.
    ///
    /// [mm]: https://doc.rust-lang.org/stable/std/sync/struct.Arc.html#method.make_mut
    pub fn make_mut(&mut self) -> &mut T
    where
        T: Clone,
    {
        let inner = Arc::make_mut(&mut self.0);
        // Clear the cache as mutation through the returned `&mut T` may invalidate it
        *inner.cached_hash.get_mut() = 0;
        &mut inner.value
    }

    pub fn ptr_eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl<T> std::ops::Deref for Harc<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0.value
    }
}

impl<T: Eq> Eq for Harc<T> {}

impl<T: PartialEq> PartialEq for Harc<T> {
    fn eq(&self, other: &Self) -> bool {
        self.ptr_eq(other) // fast path
        || self.0.value == other.0.value // cached_hash not included
    }
}

/// Produces the hash of the cached hash, different from `<T as Hash>`.
impl<T: Hash> Hash for Harc<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let mut hash = self.0.cached_hash.load(Ordering::Relaxed);
        if hash != CACHED_HASH_NOT_COMPUTED_YET {
            // cache hit
        } else {
            // It is possible for multiple threads to race and take this path for the same `HarcInner`.
            // This is ok as they should compute the same result.
            // We save on the extra space that `OnceLock<u64>` would occupy,
            // at the cost of extra computation in the unlikely case of this race.
            #[cold]
            #[inline(never)]
            fn slow_path<T: Hash>(inner: &HarcInner<T>) -> u64 {
                /// We share a single `BuildHasher` process-wide,
                /// not only for the race described above but also
                /// so that multiple `HarcInner`’s with the same contents have the same hash.
                static SHARED_RANDOM: OnceLock<RandomState> = OnceLock::new();
                let mut hasher = SHARED_RANDOM.get_or_init(RandomState::new).build_hasher();
                inner.value.hash(&mut hasher);
                let mut hash = hasher.finish();
                // Don’t use the marker value for an actual hash
                if hash == CACHED_HASH_NOT_COMPUTED_YET {
                    hash += 1
                }
                inner.cached_hash.store(hash, Ordering::Relaxed);
                hash
            }
            hash = slow_path(&self.0)
        }
        hash.hash(state);
    }
}
