//! Type aliases for hashing-based collections configured with a specific hasher,
//! as used in various places thorough the API

/// [`indexmap::IndexMap`] configured with a specific hasher
pub type IndexMap<K, V> = indexmap::IndexMap<K, V, ahash::RandomState>;

/// [`indexmap::IndexSet`] configured with a specific hasher
pub type IndexSet<T> = indexmap::IndexSet<T, ahash::RandomState>;

/// [`std::collections::HashMap`] configured with a specific hasher
pub type HashMap<K, V> = std::collections::HashMap<K, V, ahash::RandomState>;

/// [`std::collections::HashSet`] configured with a specific hasher
pub type HashSet<T> = std::collections::HashSet<T, ahash::RandomState>;
