//! Type aliases for hashing-based collections configured with a specific hasher,
//! as used in various places thorough the API

use std::hash::DefaultHasher;
use std::hash::Hash;
use std::hash::Hasher;

/// [`indexmap::IndexMap`] configured with a specific hasher
pub type IndexMap<K, V> = indexmap::IndexMap<K, V, ahash::RandomState>;

/// [`indexmap::IndexSet`] configured with a specific hasher
pub type IndexSet<T> = indexmap::IndexSet<T, ahash::RandomState>;

/// [`std::collections::HashMap`] configured with a specific hasher
pub type HashMap<K, V> = std::collections::HashMap<K, V, ahash::RandomState>;

/// [`std::collections::HashSet`] configured with a specific hasher
pub type HashSet<T> = std::collections::HashSet<T, ahash::RandomState>;

/// Order-independent hash for collections whose `PartialEq` is order-independent
/// (e.g. `IndexMap`, `IndexSet`). Uses commutative XOR of per-element hashes.
pub(crate) fn hash_unordered<H: Hasher, T: Hash>(
    items: impl Iterator<Item = T>,
    state: &mut H,
    len: usize,
) {
    len.hash(state);
    let mut combined = 0u64;
    for item in items {
        let mut h = DefaultHasher::new();
        item.hash(&mut h);
        combined ^= h.finish();
    }
    combined.hash(state);
}
