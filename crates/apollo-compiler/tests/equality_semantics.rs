use apollo_compiler::ExecutableDocument;
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
fn directive_definition_arguments_are_order_independent() {
    let schema_a = Schema::parse_and_validate(
        "type Query { field: String }
         directive @example(a: Int, b: String) on FIELD_DEFINITION",
        "a.graphql",
    )
    .unwrap();

    let schema_b = Schema::parse_and_validate(
        "type Query { field: String }
         directive @example(b: String, a: Int) on FIELD_DEFINITION",
        "b.graphql",
    )
    .unwrap();

    assert_eq!(schema_a, schema_b);
}

#[test]
fn field_definition_arguments_are_order_independent() {
    let schema_a = Schema::parse_and_validate(
        "type Query { field(x: Int, y: String): String }",
        "a.graphql",
    )
    .unwrap();

    let schema_b = Schema::parse_and_validate(
        "type Query { field(y: String, x: Int): String }",
        "b.graphql",
    )
    .unwrap();

    assert_eq!(schema_a, schema_b);
}

#[test]
fn applied_directive_arguments_are_order_independent() {
    let schema = Schema::parse_and_validate(
        "type Query { field: String }
         directive @example(a: Int, b: String) on QUERY",
        "schema.graphql",
    )
    .unwrap();

    let doc_a = ExecutableDocument::parse_and_validate(
        &schema,
        "query @example(a: 1, b: \"hello\") { field }",
        "a.graphql",
    )
    .unwrap();

    let doc_b = ExecutableDocument::parse_and_validate(
        &schema,
        "query @example(b: \"hello\", a: 1) { field }",
        "b.graphql",
    )
    .unwrap();

    assert_eq!(doc_a, doc_b);
}

#[test]
fn executable_field_arguments_are_order_independent() {
    let schema = Schema::parse_and_validate(
        "type Query { field(a: Int, b: String): String }",
        "schema.graphql",
    )
    .unwrap();

    let doc_a = ExecutableDocument::parse_and_validate(
        &schema,
        "{ field(a: 1, b: \"hello\") }",
        "a.graphql",
    )
    .unwrap();

    let doc_b = ExecutableDocument::parse_and_validate(
        &schema,
        "{ field(b: \"hello\", a: 1) }",
        "b.graphql",
    )
    .unwrap();

    assert_eq!(doc_a, doc_b);
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

#[test]
fn selection_sets_are_order_dependent() {
    let schema =
        Schema::parse_and_validate("type Query { name: String, age: Int }", "schema.graphql")
            .unwrap();

    let doc_a =
        ExecutableDocument::parse_and_validate(&schema, "{ name age }", "a.graphql").unwrap();

    let doc_b =
        ExecutableDocument::parse_and_validate(&schema, "{ age name }", "b.graphql").unwrap();

    assert_ne!(doc_a, doc_b);
}
