use apollo_compiler::Schema;

#[test]
fn directive_definition_locations_are_order_independent() {
    let schema_a = Schema::parse_and_validate(
        "type Query { field: String }
         directive @example on FIELD_DEFINITION | OBJECT",
        "a.graphql",
    )
    .unwrap();

    let schema_b = Schema::parse_and_validate(
        "type Query { field: String }
         directive @example on OBJECT | FIELD_DEFINITION",
        "b.graphql",
    )
    .unwrap();

    assert_eq!(schema_a, schema_b);
}

#[test]
fn directive_arguments_are_order_independent() {
    let schema_a = Schema::parse_and_validate(
        "directive @example(a: Int, b: Int) on FIELD_DEFINITION
         type Query { field: String @example(a: 1, b: 2) }",
        "a.graphql",
    )
    .unwrap();

    let schema_b = Schema::parse_and_validate(
        "directive @example(a: Int, b: Int) on FIELD_DEFINITION
         type Query { field: String @example(b: 2, a: 1) }",
        "b.graphql",
    )
    .unwrap();

    assert_eq!(schema_a, schema_b);
}

#[test]
fn field_definition_arguments_are_order_independent() {
    let schema_a = Schema::parse_and_validate(
        "type Query { field(a: Int, b: String): String }",
        "a.graphql",
    )
    .unwrap();

    let schema_b = Schema::parse_and_validate(
        "type Query { field(b: String, a: Int): String }",
        "b.graphql",
    )
    .unwrap();

    assert_eq!(schema_a, schema_b);
}

#[test]
fn applied_directives_are_order_dependent() {
    let schema_a = Schema::parse_and_validate(
        "directive @auth on FIELD_DEFINITION
         directive @cache on FIELD_DEFINITION
         type Query { field: String @auth @cache }",
        "a.graphql",
    )
    .unwrap();

    let schema_b = Schema::parse_and_validate(
        "directive @auth on FIELD_DEFINITION
         directive @cache on FIELD_DEFINITION
         type Query { field: String @cache @auth }",
        "b.graphql",
    )
    .unwrap();

    assert_ne!(schema_a, schema_b);
}
