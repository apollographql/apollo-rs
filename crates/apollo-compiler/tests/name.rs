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

/// Maps keyed by `Name` are looked up by `&str` throughout the API, so both
/// must hash the same way with the hasher the collections use.
#[test]
fn collections_find_name_keys_by_str() {
    let mut map = apollo_compiler::collections::IndexMap::default();
    map.insert(Name::new("getName").unwrap(), 1);
    assert_eq!(map.get("getName"), Some(&1));

    let mut set = apollo_compiler::collections::HashSet::default();
    set.insert(Name::new("getName").unwrap());
    assert!(set.contains("getName"));
}
