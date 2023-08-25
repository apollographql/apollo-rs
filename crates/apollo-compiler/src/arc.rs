use std::hash::Hash;
use std::hash::Hasher;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;

use std::collections::hash_map::RandomState;
use std::fmt;
use std::hash::BuildHasher;
use std::sync::OnceLock;

/// A thread-safe reference-counted smart pointer that caches `Hash`.
///
/// Weak references are not supported.
///
/// For the cache to be correct, **`T` is expected to have a stable hash**
/// a long as no `&mut T` exclusive reference to it is given out.
/// Generally this excludes interior mutability.
///
/// This `Arc` cannot implement [`Borrow<T>`][std::borrow::Borrow] because `Arc<T> as Hash`
/// produces a result (the hash of the cached hash) different from `T as Hash`.
pub struct Arc<T>(triomphe::Arc<ArcInner<T>>);

#[derive(Clone)]
struct ArcInner<T> {
    hash_cache: HashCache,
    value: T,
}

pub(crate) struct HashCache(AtomicU64);

impl<T> Arc<T> {
    pub fn new(value: T) -> Self {
        Self(triomphe::Arc::new(ArcInner {
            hash_cache: HashCache::new(),
            value,
        }))
    }

    /// Returns whether two `Arc`s point to the same memory allocation.
    pub fn ptr_eq(&self, other: &Self) -> bool {
        triomphe::Arc::ptr_eq(&self.0, &other.0)
    }

    /// Returns a mutable reference to `T`, cloning it if necessary
    ///
    /// This is functionally equivalent to [`Arc::make_mut`][mm] from the standard library.
    ///
    /// If this `Arc` is uniquely owned, `make_mut()` will provide a mutable
    /// reference to the contents. If not, `make_mut()` will create a _new_ `Arc`
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
        let inner = triomphe::Arc::make_mut(&mut self.0);
        // Clear the cache as mutation through the returned `&mut T` may invalidate it
        inner.hash_cache.clear();
        &mut inner.value
    }

    /// Returns a mutable reference to `T` if this `Arc` is uniquely owned
    pub fn get_mut(&mut self) -> Option<&mut T> {
        triomphe::Arc::get_mut(&mut self.0).map(|inner| &mut inner.value)
    }
}

impl<T> std::ops::Deref for Arc<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0.value
    }
}

impl<T> Clone for Arc<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T: Eq> Eq for Arc<T> {}

impl<T: PartialEq> PartialEq for Arc<T> {
    fn eq(&self, other: &Self) -> bool {
        self.ptr_eq(other) // fast path
        || self.0.value == other.0.value // hash cache not included
    }
}

impl<T: Hash> Hash for Arc<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash_cache.get(&self.0.value).hash(state)
    }
}

impl HashCache {
    const NOT_COMPUTED_YET: u64 = 0;

    pub(crate) fn new() -> Self {
        Self(AtomicU64::new(Self::NOT_COMPUTED_YET))
    }

    pub(crate) fn clear(&mut self) {
        *self.0.get_mut() = Self::NOT_COMPUTED_YET
    }

    pub(crate) fn get<T: ?Sized + Hash>(&self, value: &T) -> u64 {
        let hash = self.0.load(Ordering::Relaxed);
        if hash != Self::NOT_COMPUTED_YET {
            // cache hit
            hash
        } else {
            self.get_slow_path(value)
        }
    }

    // It is possible for multiple threads to race and take this path for the same `HarcInner`.
    // This is ok as they should compute the same result.
    // We save on the extra space that `OnceLock<u64>` would occupy,
    // at the cost of extra computation in the unlikely case of this race.
    #[cold]
    #[inline(never)]
    fn get_slow_path<T: ?Sized + Hash>(&self, value: &T) -> u64 {
        /// We share a single `BuildHasher` process-wide,
        /// not only for the race described above but also
        /// so that multiple `HarcInner`’s with the same contents have the same hash.
        static SHARED_RANDOM: OnceLock<RandomState> = OnceLock::new();
        let mut hasher = SHARED_RANDOM.get_or_init(RandomState::new).build_hasher();
        value.hash(&mut hasher);
        let mut hash = hasher.finish();
        // Don’t use the marker value for an actual hash
        if hash == Self::NOT_COMPUTED_YET {
            hash += 1
        }
        self.0.store(hash, Ordering::Relaxed);
        hash
    }
}

impl Clone for HashCache {
    fn clone(&self) -> Self {
        Self(AtomicU64::new(self.0.load(Ordering::Relaxed)))
    }
}

impl<T: fmt::Debug> fmt::Debug for Arc<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.value.fmt(f)
    }
}

impl<T: Default> Default for Arc<T> {
    fn default() -> Self {
        Self::new(Default::default())
    }
}

impl<T> AsRef<T> for Arc<T> {
    fn as_ref(&self) -> &T {
        self
    }
}

impl AsRef<str> for Arc<String> {
    fn as_ref(&self) -> &str {
        self
    }
}
