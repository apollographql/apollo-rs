use apollo_compiler::name;
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
fn node_name_set_lookup() {
    use apollo_compiler::Node;
    // Enough entries that indexmap hashes instead of scanning linearly.
    let mut set = apollo_compiler::collections::IndexSet::default();
    for i in 0..16 {
        set.insert(Node::new(Name::new(&format!("nodeSetLookup{i}")).unwrap()));
    }
    assert!(set.contains(&name!("nodeSetLookup3")));
    assert!(set.contains(&Node::new(name!("nodeSetLookup3"))));
    assert!(!set.contains(&name!("nodeSetLookup16")));
}
