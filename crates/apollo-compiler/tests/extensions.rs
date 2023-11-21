use apollo_compiler::Schema;

#[test]
fn test_orphan_extensions() {
    let input = r#"
        extend schema @dir { query: Q }
        extend type Obj @dir
        directive @dir on SCHEMA | OBJECT
        type Q { x: Int }
    "#;

    // By default, orphan extensions are errors:
    let schema = Schema::parse(input, "schema.graphql");
    assert!(!schema.schema_definition.directives.has("dir"));
    assert!(!schema.types.contains_key("Obj"));
    let err = schema
        .validate(Default::default())
        .unwrap_err()
        .to_string_no_color();
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
        .build();
    assert!(schema2.schema_definition.directives.has("dir"));
    assert!(schema2.types["Obj"].directives().has("dir"));
    schema2.validate(Default::default()).unwrap();
}

#[test]
fn test_orphan_extensions_kind_mismatch() {
    let input = r#"
    extend type T @dir
    extend interface T @dir
    directive @dir repeatable on SCHEMA | OBJECT
"#;

    let schema = Schema::builder()
        .adopt_orphan_extensions()
        .parse(input, "schema.graphql")
        .build();
    let type_def = &schema.types["T"];
    assert!(type_def.is_object());
    assert_eq!(type_def.directives().get_all("dir").count(), 1);
    let err = schema
        .validate(Default::default())
        .unwrap_err()
        .to_string_no_color();
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

    let schema = Schema::parse(input, "schema.graphql");
    schema.validate(Default::default()).unwrap();
    assert!(schema.schema_definition.directives.has("dir"));
}
