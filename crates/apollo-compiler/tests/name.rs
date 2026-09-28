use apollo_compiler::Name;

/// cargo +nightly miri test --test main -- name::smoke_test
#[test]
fn smoke_test() {
    let heap = Name::new("abc").unwrap();
    let static_ = Name::new_static("abc").unwrap();
    let heap_2 = heap.clone();
    let static_2 = static_.clone();
    assert_eq!(heap_2.as_str(), static_2.as_str());
    assert_eq!(heap_2, static_2);
}

#[test]
fn frozen_interning_name_semantics() {
    use apollo_compiler::NameKey;
    use std::collections::hash_map::DefaultHasher;
    use std::hash::Hash;
    use std::hash::Hasher;

    fn hash_of(value: &impl Hash) -> u64 {
        let mut hasher = DefaultHasher::new();
        value.hash(&mut hasher);
        hasher.finish()
    }

    // Interned pre-freeze (as schema names are during schema building).
    let schema_name = Name::new("frozenTestSchemaName").unwrap();
    apollo_compiler::freeze_interning();

    // Post-freeze: same string still resolves to the same symbol.
    let same = Name::new("frozenTestSchemaName").unwrap();
    assert_eq!(schema_name, same);
    assert_eq!(hash_of(&schema_name), hash_of(&same));

    // Post-freeze misses (hostile-alias shaped): equality and hashing fall
    // back to strings, consistently between instances.
    let miss_a = Name::new("frozenTestNotInterned").unwrap();
    let miss_b = Name::new("frozenTestNotInterned").unwrap();
    assert_eq!(miss_a, miss_b);
    assert_eq!(hash_of(&miss_a), hash_of(&miss_b));
    // Mixed comparison: interned vs missed are never equal.
    assert_ne!(schema_name, miss_a);

    // Maps keyed by both kinds of name work, including NameKey probes.
    let mut map = apollo_compiler::collections::IndexMap::default();
    map.insert(schema_name.clone(), 1);
    map.insert(miss_a.clone(), 2);
    assert_eq!(map.get(&same), Some(&1));
    assert_eq!(map.get(&miss_b), Some(&2));
    assert_eq!(map.get(&NameKey("frozenTestSchemaName")), Some(&1));
    assert_eq!(map.get(&NameKey("frozenTestNotInterned")), Some(&2));
    assert_eq!(map.get(&NameKey("frozenTestAbsent")), None);
    // Literal names hash like the interned name with the same string.
    assert_eq!(
        map.get(&apollo_compiler::name!("frozenTestSchemaName")),
        Some(&1)
    );
    assert_eq!(
        map.get(&apollo_compiler::name!("frozenTestNotInterned")),
        Some(&2)
    );

    // Literal names are not interned and compare by string with both kinds.
    assert_eq!(apollo_compiler::name!("frozenTestSchemaName"), schema_name);
    assert_eq!(apollo_compiler::name!("frozenTestNotInterned"), miss_a);

    // An `Arc` whose string is already interned is swapped for the table's copy.
    let from_arc = Name::try_from(std::sync::Arc::<str>::from("frozenTestSchemaName")).unwrap();
    assert_eq!(from_arc.as_static_str(), schema_name.as_static_str());
    assert!(from_arc.as_static_str().is_some());
}

#[test]
fn node_name_set_lookup() {
    use apollo_compiler::NameKey;
    use apollo_compiler::Node;
    // Enough entries that indexmap hashes instead of scanning linearly.
    let mut set = apollo_compiler::collections::IndexSet::default();
    for i in 0..16 {
        set.insert(Node::new(Name::new(&format!("nodeSetLookup{i}")).unwrap()));
    }
    assert!(set.contains(&NameKey("nodeSetLookup3")));
    assert!(set.contains(&Node::new(apollo_compiler::name!("nodeSetLookup3"))));
    assert!(!set.contains(&NameKey("nodeSetLookup16")));
}
