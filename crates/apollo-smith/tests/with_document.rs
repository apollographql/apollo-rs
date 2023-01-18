/// Non-regression for https://github.com/apollographql/apollo-rs/issues/426
#[cfg(feature = "parser-impl")]
#[test]
fn test_with_document() {
    let mut u = arbitrary::Unstructured::new(&[1]);
    let schema = "schema { query: Query } type Query { id: ID! }";
    let schema = apollo_parser::Parser::new(schema)
        .parse()
        .document()
        .try_into()
        .unwrap();
    apollo_smith::DocumentBuilder::with_document(&mut u, schema)
        .unwrap()
        .operation_definition()
        .unwrap();
}
