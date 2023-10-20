use apollo_compiler::NodeStr;

/// cargo +nightly miri test --test main -- node_str
#[test]
fn smoke_test() {
    let heap = NodeStr::new(&123.to_string());
    let static_ = NodeStr::from_static(&"123");
    let heap_2 = heap.clone();
    let static_2 = static_.clone();
    assert_eq!(heap_2.as_str(), static_2.as_str());
    assert_eq!(heap_2, static_2);
}
