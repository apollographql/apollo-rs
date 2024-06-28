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
