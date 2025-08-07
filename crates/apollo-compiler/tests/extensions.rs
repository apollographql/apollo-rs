use apollo_compiler::Schema;

fn validate_schema(schema: Schema) {
    let validated_schema = schema.validate().unwrap();

    // Test the `Schema::to_string()` with orphan extensions by parsing and validating the printed
    // schema.
    let printed_schema = validated_schema.to_string();
    Schema::builder()
        .adopt_orphan_extensions()
        .parse(printed_schema, "printed_schema.graphql")
        .build()
        .unwrap()
        .validate()
        .unwrap();
}

#[test]
fn test_orphan_extensions() {
    let input = r#"
        extend schema @dir { query: Q }
        extend type Obj @dir { foo: String }
        extend interface I @dir { foo: String }
        extend union U @dir = Obj
        extend enum E @dir { FOO, BAR }
        extend input Input @dir { bar: String }
        directive @dir on SCHEMA | SCALAR | OBJECT | INTERFACE | UNION | ENUM | INPUT_OBJECT
        type Q { x: Int }
    "#;

    // By default, orphan extensions are errors:
    let invalid = Schema::parse_and_validate(input, "schema.graphql").unwrap_err();
    assert!(!invalid.partial.schema_definition.directives.has("dir"));
    assert!(!invalid.partial.types.contains_key("Obj"));
    let err = invalid.errors.to_string();
    assert!(
        err.contains("schema extension without a schema definition"),
        "{err}"
    );
    assert!(
        err.contains("type extension for undefined type `Obj`"),
        "{err}"
    );

    // Opt in to non-standard behavior of adopting them instead:
    let schema2 = Schema::builder()
        .adopt_orphan_extensions()
        .parse(input, "schema.graphql")
        .build()
        .unwrap();
    assert!(schema2.schema_definition.directives.has("dir"));
    assert!(schema2.types["Obj"].directives().has("dir"));
    validate_schema(schema2);
}

#[test]
fn test_orphan_extensions_schema_with_default_query_name() {
    let input = r#"
        extend schema { query: Query }
        type Query { x: Int }
    "#;

    let schema = Schema::builder()
        .adopt_orphan_extensions()
        .parse(input, "schema.graphql")
        .build()
        .unwrap();

    validate_schema(schema);
}

#[test]
fn test_orphan_extensions_schema_def_with_extensions() {
    let input = r#"
        extend schema { query: Query }
        extend schema { subscription: S }
        schema { mutation: Mutation }
        type Query { x: Int }
        type Mutation { y: Int }
        type S { z: Int }
    "#;

    let schema = Schema::builder()
        .adopt_orphan_extensions()
        .parse(input, "schema.graphql")
        .build()
        .unwrap();

    validate_schema(schema);
}

#[test]
fn test_invalid_orphan_extensions_schema_def_with_duplicate_root_operation() {
    let input = r#"
        extend schema { query: Query }
        extend schema { subscription: S }
        schema { mutation: Mutation, query: AnotherQuery }
        type Query { x: Int }
        type Mutation { y: Int }
        type S { z: Int }
        type AnotherQuery { a: String }
    "#;

    let invalid = Schema::builder()
        .adopt_orphan_extensions()
        .parse(input, "schema.graphql")
        .build()
        .unwrap_err();

    let err = invalid.errors.to_string();
    assert!(err.contains("duplicate definitions for the `query` root operation type"));
}

#[test]
fn test_orphan_schema_extension_with_root_type_disables_implicit_root_types() {
    let input = r#"
        extend schema { query: Query }
        type Query { viruses: [Virus] }
        type Virus { mutations: [Mutation] }
        type Mutation { something: String }
    "#;

    let schema = Schema::builder()
        .adopt_orphan_extensions()
        .parse(input, "schema.graphql")
        .build()
        .unwrap();

    assert!(schema.schema_definition.mutation.is_none());
    validate_schema(schema);
}

#[test]
fn test_orphan_schema_extension_without_root_type_enables_implicit_root_types() {
    let input = r#"
        directive @something on SCHEMA
        extend schema @something
        type Query { field: Int }
    "#;

    let schema = Schema::builder()
        .adopt_orphan_extensions()
        .parse(input, "schema.graphql")
        .build()
        .unwrap();

    assert!(schema.schema_definition.query.is_some());
    validate_schema(schema);
}

#[test]
fn test_orphan_schema_extension_with_directive_application() {
    let input = r#"
        directive @something on SCHEMA
        extend schema @something { query: Query }
        type Query { field: Int }
    "#;

    let schema = Schema::builder()
        .adopt_orphan_extensions()
        .parse(input, "schema.graphql")
        .build()
        .unwrap();

    assert!(schema.schema_definition.query.is_some());
    validate_schema(schema);
}

#[test]
fn test_orphan_extensions_kind_mismatch() {
    let input = r#"
    extend type T @dir
    extend interface T @dir
    directive @dir repeatable on SCHEMA | OBJECT
"#;

    let invalid = Schema::builder()
        .adopt_orphan_extensions()
        .parse(input, "schema.graphql")
        .build()
        .unwrap_err();
    let type_def = &invalid.partial.types["T"];
    assert!(type_def.is_object());
    assert_eq!(type_def.directives().get_all("dir").count(), 1);
    let err = invalid.errors.to_string();
    assert!(
        err.contains("adding an interface type extension, but `T` is an object type"),
        "{err}"
    );
}

/// https://github.com/apollographql/apollo-rs/issues/682
#[test]
fn test_extend_implicit_schema() {
    let input = r#"
    type Query { field: Int } # creates an implicit schema definition that can be extended
    extend schema @dir
    directive @dir on SCHEMA
"#;

    let schema = Schema::parse_and_validate(input, "schema.graphql").unwrap();
    assert!(schema.schema_definition.directives.has("dir"));
}
