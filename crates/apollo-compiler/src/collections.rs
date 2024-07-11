pub mod fast {
    use indexmap::IndexMap as IM;
    use indexmap::IndexSet as IS;

    pub type IndexMap<K, V> = IM<K, V, ahash::RandomState>;
    pub type IndexSet<T> = IS<T, ahash::RandomState>;
    pub type HashSet<T> = std::collections::HashSet<T, ahash::RandomState>;
}
