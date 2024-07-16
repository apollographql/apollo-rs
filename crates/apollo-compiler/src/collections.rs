//! Type aliases for hashing-based collections configured with a specific hasher,
//! as used in various places thorough the API

use indexmap::IndexMap as IM;
use indexmap::IndexSet as IS;

pub type IndexMap<K, V> = IM<K, V, ahash::RandomState>;
pub type IndexSet<T> = IS<T, ahash::RandomState>;
pub type HashMap<K, V> = std::collections::HashMap<K, V, ahash::RandomState>;
pub type HashSet<T> = std::collections::HashSet<T, ahash::RandomState>;
