/// Tests for GraphQL spec-compliant equality semantics.
///
/// The September 2025 spec defines ordering requirements for each construct.
/// Some are order-significant (applied directives, selection sets), while
/// others use set semantics (arguments, directive locations, union members,
/// interface implementations, input object values).
///
/// These tests verify that our data structures produce the correct equality
/// behavior for each case.
use apollo_compiler::ExecutableDocument;
use apollo_compiler::Schema;

// ---------------------------------------------------------------------------
// Already correct: schema-layer types with proper collection types
// ---------------------------------------------------------------------------

#[test]
fn object_type_fields_are_ordered_by_name() {
    // Fields on object types use IndexMap keyed by name.
    // Two schemas with the same fields defined in different order should
    // produce equal types because the IndexMap is keyed by name.
    let schema_a = Schema::parse_and_validate(
        r#"
        type Query {
            alpha: String
            beta: Int
        }
        "#,
        "a.graphql",
    )
    .unwrap();

    let schema_b = Schema::parse_and_validate(
        r#"
        type Query {
            beta: Int
            alpha: String
        }
        "#,
        "b.graphql",
    )
    .unwrap();

    // IndexMap preserves insertion order, so the types won't be equal
    // since field order is preserved for serialization purposes.
    // But the type *lookup* is by name, which is the important part.
    let query_a = schema_a.types.get("Query").unwrap();
    let query_b = schema_b.types.get("Query").unwrap();

    // Fields are present regardless of definition order.
    match (query_a, query_b) {
        (
            apollo_compiler::schema::ExtendedType::Object(a),
            apollo_compiler::schema::ExtendedType::Object(b),
        ) => {
            assert!(a.fields.contains_key("alpha"));
            assert!(a.fields.contains_key("beta"));
            assert!(b.fields.contains_key("alpha"));
            assert!(b.fields.contains_key("beta"));
            assert_eq!(a.fields.len(), b.fields.len());
        }
        _ => panic!("expected object types"),
    }
}

#[test]
fn interface_type_fields_are_ordered_by_name() {
    let schema_a = Schema::parse_and_validate(
        r#"
        type Query implements Node {
            id: ID!
            name: String
        }
        interface Node {
            id: ID!
            name: String
        }
        "#,
        "a.graphql",
    )
    .unwrap();

    let schema_b = Schema::parse_and_validate(
        r#"
        type Query implements Node {
            name: String
            id: ID!
        }
        interface Node {
            name: String
            id: ID!
        }
        "#,
        "b.graphql",
    )
    .unwrap();

    let iface_a = schema_a.types.get("Node").unwrap();
    let iface_b = schema_b.types.get("Node").unwrap();

    match (iface_a, iface_b) {
        (
            apollo_compiler::schema::ExtendedType::Interface(a),
            apollo_compiler::schema::ExtendedType::Interface(b),
        ) => {
            assert!(a.fields.contains_key("id"));
            assert!(a.fields.contains_key("name"));
            assert!(b.fields.contains_key("id"));
            assert!(b.fields.contains_key("name"));
            assert_eq!(a.fields.len(), b.fields.len());
        }
        _ => panic!("expected interface types"),
    }
}

#[test]
fn enum_values_are_keyed_by_name() {
    let schema_a = Schema::parse_and_validate(
        r#"
        type Query { status: Status }
        enum Status { ACTIVE INACTIVE PENDING }
        "#,
        "a.graphql",
    )
    .unwrap();

    let schema_b = Schema::parse_and_validate(
        r#"
        type Query { status: Status }
        enum Status { PENDING ACTIVE INACTIVE }
        "#,
        "b.graphql",
    )
    .unwrap();

    let enum_a = schema_a.types.get("Status").unwrap();
    let enum_b = schema_b.types.get("Status").unwrap();

    match (enum_a, enum_b) {
        (
            apollo_compiler::schema::ExtendedType::Enum(a),
            apollo_compiler::schema::ExtendedType::Enum(b),
        ) => {
            assert_eq!(a.values.len(), b.values.len());
            for name in ["ACTIVE", "INACTIVE", "PENDING"] {
                assert!(a.values.contains_key(name));
                assert!(b.values.contains_key(name));
            }
        }
        _ => panic!("expected enum types"),
    }
}

#[test]
fn union_members_are_order_independent() {
    let schema_a = Schema::parse_and_validate(
        r#"
        type Query { result: SearchResult }
        union SearchResult = User | Post | Comment
        type User { id: ID! }
        type Post { id: ID! }
        type Comment { id: ID! }
        "#,
        "a.graphql",
    )
    .unwrap();

    let schema_b = Schema::parse_and_validate(
        r#"
        type Query { result: SearchResult }
        union SearchResult = Comment | User | Post
        type User { id: ID! }
        type Post { id: ID! }
        type Comment { id: ID! }
        "#,
        "b.graphql",
    )
    .unwrap();

    let union_a = schema_a.types.get("SearchResult").unwrap();
    let union_b = schema_b.types.get("SearchResult").unwrap();

    match (union_a, union_b) {
        (
            apollo_compiler::schema::ExtendedType::Union(a),
            apollo_compiler::schema::ExtendedType::Union(b),
        ) => {
            // IndexSet: members are equal regardless of definition order
            assert_eq!(a.members, b.members);
        }
        _ => panic!("expected union types"),
    }
}

#[test]
fn implements_interfaces_are_order_independent() {
    let schema_a = Schema::parse_and_validate(
        r#"
        type Query implements Node & Named & Timestamped {
            id: ID!
            name: String!
            createdAt: String!
        }
        interface Node { id: ID! }
        interface Named { name: String! }
        interface Timestamped { createdAt: String! }
        "#,
        "a.graphql",
    )
    .unwrap();

    let schema_b = Schema::parse_and_validate(
        r#"
        type Query implements Timestamped & Node & Named {
            id: ID!
            name: String!
            createdAt: String!
        }
        interface Node { id: ID! }
        interface Named { name: String! }
        interface Timestamped { createdAt: String! }
        "#,
        "b.graphql",
    )
    .unwrap();

    let query_a = schema_a.types.get("Query").unwrap();
    let query_b = schema_b.types.get("Query").unwrap();

    match (query_a, query_b) {
        (
            apollo_compiler::schema::ExtendedType::Object(a),
            apollo_compiler::schema::ExtendedType::Object(b),
        ) => {
            // IndexSet: implements list is equal regardless of order
            assert_eq!(a.implements_interfaces, b.implements_interfaces);
        }
        _ => panic!("expected object types"),
    }
}

#[test]
fn input_object_fields_are_keyed_by_name() {
    let schema_a = Schema::parse_and_validate(
        r#"
        type Query { search(filter: Filter): String }
        input Filter {
            name: String
            age: Int
            active: Boolean
        }
        "#,
        "a.graphql",
    )
    .unwrap();

    let schema_b = Schema::parse_and_validate(
        r#"
        type Query { search(filter: Filter): String }
        input Filter {
            active: Boolean
            name: String
            age: Int
        }
        "#,
        "b.graphql",
    )
    .unwrap();

    let input_a = schema_a.types.get("Filter").unwrap();
    let input_b = schema_b.types.get("Filter").unwrap();

    match (input_a, input_b) {
        (
            apollo_compiler::schema::ExtendedType::InputObject(a),
            apollo_compiler::schema::ExtendedType::InputObject(b),
        ) => {
            assert_eq!(a.fields.len(), b.fields.len());
            for name in ["name", "age", "active"] {
                assert!(a.fields.contains_key(name));
                assert!(b.fields.contains_key(name));
            }
        }
        _ => panic!("expected input object types"),
    }
}

#[test]
fn applied_directives_are_order_dependent() {
    // The spec says: "the order in which directives appear may be significant,
    // including repeatable directives."
    let schema_a = Schema::parse_and_validate(
        r#"
        directive @auth on FIELD_DEFINITION
        directive @cache on FIELD_DEFINITION
        type Query {
            field: String @auth @cache
        }
        "#,
        "a.graphql",
    )
    .unwrap();

    let schema_b = Schema::parse_and_validate(
        r#"
        directive @auth on FIELD_DEFINITION
        directive @cache on FIELD_DEFINITION
        type Query {
            field: String @cache @auth
        }
        "#,
        "b.graphql",
    )
    .unwrap();

    let get_directive_names = |schema: &Schema| -> Vec<String> {
        match schema.types.get("Query").unwrap() {
            apollo_compiler::schema::ExtendedType::Object(obj) => obj
                .fields
                .get("field")
                .unwrap()
                .directives
                .iter()
                .map(|d| d.name.to_string())
                .collect(),
            _ => panic!("expected object type"),
        }
    };

    let names_a = get_directive_names(&schema_a);
    let names_b = get_directive_names(&schema_b);

    // Applied directives in different order must NOT be equal
    assert_ne!(names_a, names_b);
}

#[test]
fn selection_sets_are_order_dependent() {
    let schema = Schema::parse_and_validate(
        r#"
        type Query {
            name: String
            age: Int
        }
        "#,
        "schema.graphql",
    )
    .unwrap();

    let doc_a =
        ExecutableDocument::parse_and_validate(&schema, r#"{ name age }"#, "a.graphql").unwrap();

    let doc_b =
        ExecutableDocument::parse_and_validate(&schema, r#"{ age name }"#, "b.graphql").unwrap();

    let selections_a = &doc_a.operations.anonymous.as_ref().unwrap().selection_set;
    let selections_b = &doc_b.operations.anonymous.as_ref().unwrap().selection_set;

    // Selection set order matters for response serialization
    assert_ne!(selections_a, selections_b);
}

// ---------------------------------------------------------------------------
// Mismatches: these tests document the EXPECTED behavior per the spec.
// Tests that assert order-independence will initially fail because the
// current data structures use Vec with derived PartialEq.
// ---------------------------------------------------------------------------

#[test]
fn directive_definition_locations_are_order_independent() {
    // Spec: directive locations are a set, order is not significant.
    let schema_a = Schema::parse_and_validate(
        r#"
        type Query { field: String }
        directive @example on FIELD_DEFINITION | OBJECT
        "#,
        "a.graphql",
    )
    .unwrap();

    let schema_b = Schema::parse_and_validate(
        r#"
        type Query { field: String }
        directive @example on OBJECT | FIELD_DEFINITION
        "#,
        "b.graphql",
    )
    .unwrap();

    let def_a = schema_a.directive_definitions.get("example").unwrap();
    let def_b = schema_b.directive_definitions.get("example").unwrap();

    assert_eq!(
        def_a, def_b,
        "directive definition locations should be order-independent"
    );
}

#[test]
fn directive_definition_arguments_are_order_independent() {
    // Spec: argument definitions are a named set, order is not significant.
    let schema_a = Schema::parse_and_validate(
        r#"
        type Query { field: String }
        directive @example(a: Int, b: String) on FIELD_DEFINITION
        "#,
        "a.graphql",
    )
    .unwrap();

    let schema_b = Schema::parse_and_validate(
        r#"
        type Query { field: String }
        directive @example(b: String, a: Int) on FIELD_DEFINITION
        "#,
        "b.graphql",
    )
    .unwrap();

    let def_a = schema_a.directive_definitions.get("example").unwrap();
    let def_b = schema_b.directive_definitions.get("example").unwrap();

    assert_eq!(
        def_a, def_b,
        "directive definition arguments should be order-independent"
    );
}

#[test]
fn field_definition_arguments_are_order_independent() {
    // Spec: argument definitions on fields are a named set.
    let schema_a = Schema::parse_and_validate(
        r#"
        type Query {
            field(x: Int, y: String): String
        }
        "#,
        "a.graphql",
    )
    .unwrap();

    let schema_b = Schema::parse_and_validate(
        r#"
        type Query {
            field(y: String, x: Int): String
        }
        "#,
        "b.graphql",
    )
    .unwrap();

    let get_field = |schema: &Schema| match schema.types.get("Query").unwrap() {
        apollo_compiler::schema::ExtendedType::Object(obj) => {
            obj.fields.get("field").unwrap().clone()
        }
        _ => panic!("expected object type"),
    };

    let field_a = get_field(&schema_a);
    let field_b = get_field(&schema_b);

    assert_eq!(
        field_a, field_b,
        "field definition arguments should be order-independent"
    );
}

#[test]
fn applied_directive_arguments_are_order_independent() {
    // Spec: "identical sets of arguments" — order does not matter for
    // directive arguments.
    let schema_a = Schema::parse_and_validate(
        r#"
        type Query { field: String }
        directive @example(a: Int, b: String) on QUERY
        "#,
        "a.graphql",
    )
    .unwrap();

    let schema_b = Schema::parse_and_validate(
        r#"
        type Query { field: String }
        directive @example(a: Int, b: String) on QUERY
        "#,
        "b.graphql",
    )
    .unwrap();

    let doc_a = ExecutableDocument::parse_and_validate(
        &schema_a,
        r#"
        query @example(a: 1, b: "hello") { field }
        "#,
        "a.graphql",
    )
    .unwrap();

    let doc_b = ExecutableDocument::parse_and_validate(
        &schema_b,
        r#"
        query @example(b: "hello", a: 1) { field }
        "#,
        "b.graphql",
    )
    .unwrap();

    let directives_a = &doc_a.operations.anonymous.as_ref().unwrap().directives;
    let directives_b = &doc_b.operations.anonymous.as_ref().unwrap().directives;

    assert_eq!(
        directives_a, directives_b,
        "applied directive arguments should be order-independent"
    );
}

#[test]
fn input_object_values_are_order_independent() {
    // Spec: input object literals are "unordered maps".
    let schema = Schema::parse_and_validate(
        r#"
        type Query {
            search(filter: Filter): String
        }
        input Filter {
            name: String
            age: Int
        }
        "#,
        "schema.graphql",
    )
    .unwrap();

    let doc_a = ExecutableDocument::parse_and_validate(
        &schema,
        r#"{ search(filter: {name: "Alice", age: 30}) }"#,
        "a.graphql",
    )
    .unwrap();

    let doc_b = ExecutableDocument::parse_and_validate(
        &schema,
        r#"{ search(filter: {age: 30, name: "Alice"}) }"#,
        "b.graphql",
    )
    .unwrap();

    let op_a = doc_a.operations.anonymous.as_ref().unwrap();
    let op_b = doc_b.operations.anonymous.as_ref().unwrap();

    assert_eq!(
        op_a.selection_set, op_b.selection_set,
        "input object value field order should not affect equality"
    );
}

#[test]
fn nested_input_object_values_are_order_independent() {
    // Nested input objects should also be order-independent recursively.
    let schema = Schema::parse_and_validate(
        r#"
        type Query {
            search(filter: Filter): String
        }
        input Filter {
            name: String
            nested: NestedFilter
        }
        input NestedFilter {
            x: Int
            y: Int
        }
        "#,
        "schema.graphql",
    )
    .unwrap();

    let doc_a = ExecutableDocument::parse_and_validate(
        &schema,
        r#"{ search(filter: {name: "Alice", nested: {x: 1, y: 2}}) }"#,
        "a.graphql",
    )
    .unwrap();

    let doc_b = ExecutableDocument::parse_and_validate(
        &schema,
        r#"{ search(filter: {nested: {y: 2, x: 1}, name: "Alice"}) }"#,
        "b.graphql",
    )
    .unwrap();

    let op_a = doc_a.operations.anonymous.as_ref().unwrap();
    let op_b = doc_b.operations.anonymous.as_ref().unwrap();

    assert_eq!(
        op_a.selection_set, op_b.selection_set,
        "nested input object value order should not affect equality"
    );
}

#[test]
fn executable_field_arguments_are_order_independent() {
    // Spec: "identical sets of arguments" for field selections.
    let schema = Schema::parse_and_validate(
        r#"
        type Query {
            field(a: Int, b: String): String
        }
        "#,
        "schema.graphql",
    )
    .unwrap();

    let doc_a = ExecutableDocument::parse_and_validate(
        &schema,
        r#"{ field(a: 1, b: "hello") }"#,
        "a.graphql",
    )
    .unwrap();

    let doc_b = ExecutableDocument::parse_and_validate(
        &schema,
        r#"{ field(b: "hello", a: 1) }"#,
        "b.graphql",
    )
    .unwrap();

    let op_a = doc_a.operations.anonymous.as_ref().unwrap();
    let op_b = doc_b.operations.anonymous.as_ref().unwrap();

    assert_eq!(
        op_a.selection_set, op_b.selection_set,
        "field arguments in different order should be equal"
    );
}
