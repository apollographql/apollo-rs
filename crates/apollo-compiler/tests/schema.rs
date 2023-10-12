use apollo_compiler::Schema;

#[test]
fn find_definitions_with_directive() {
    let schema = r#"
        type ObjectOne @key(field: "id") {
          id: ID!
          inStock: Boolean!
        }

        type ObjectTwo @key(field: "name") {
          name: String!
          address: String!
        }

        type ObjectThree {
            price: Int
        }
    "#;

    let schema = Schema::parse(schema, "schema.graphql");

    let mut key_definition_names: Vec<&str> = schema
        .types
        .iter()
        .filter(|(_name, def)| def.directives().has("key"))
        .map(|(name, _def)| name.as_str())
        .collect();
    key_definition_names.sort();
    assert_eq!(key_definition_names, ["ObjectOne", "ObjectTwo"])
}

#[test]
fn test_schema_reserialize() {
    let input = r#"
        extend type Query {
            withArg(arg: Boolean): String @deprecated,
        }

        type Query {
            int: Int,
        }

        extend type implements Inter

        interface Inter {
            string: String
        }

        extend type Query @customDirective;

        extend type Query {
            string: String,
        }

        directive @customDirective on OBJECT;
    "#;
    // Order is mostly not preserved
    let expected = expect_test::expect![
        r#"directive @customDirective on OBJECT

type Query {
  int: Int
}

extend type Query @customDirective

extend type Query {
  withArg(arg: Boolean): String @deprecated
}

extend type Query {
  string: String
}

interface Inter {
  string: String
}
"#
    ];
    let schema = Schema::parse(input, "schema.graphql");
    expected.assert_eq(&schema.to_string());
}

#[test]
fn is_subtype() {
    fn gen_schema_types(schema: &str) -> Schema {
        let base_schema = r#"
            type Query {
                me: String
            }
            type Foo {
                me: String
            }
            type Bar {
                me: String
            }
            type Baz {
                me: String
            }

            union UnionType2 = Foo | Bar
            "#;
        Schema::builder()
            .parse(SUPERGRAPH_BOILERPLATE, "boilerplate")
            .parse(base_schema, "base")
            .parse(schema, "schema")
            .build()
    }

    fn gen_schema_interfaces(schema: &str) -> Schema {
        let base_schema = r#"
            type Query {
                me: String
            }
            interface Foo {
                me: String
            }
            interface Bar {
                me: String
            }
            interface Baz {
                me: String,
            }

            type ObjectType2 implements Foo & Bar { me: String }
            interface InterfaceType2 implements Foo & Bar { me: String }
            "#;
        Schema::builder()
            .parse(SUPERGRAPH_BOILERPLATE, "boilerplate")
            .parse(base_schema, "base")
            .parse(schema, "schema")
            .build()
    }

    let schema = gen_schema_types("union UnionType = Foo | Bar | Baz");
    assert!(schema.is_subtype("UnionType", "Foo"));
    assert!(schema.is_subtype("UnionType", "Bar"));
    assert!(schema.is_subtype("UnionType", "Baz"));
    assert!(!schema.is_subtype("UnionType", "UnionType"));
    assert!(!schema.is_subtype("UnionType", "Query"));
    assert!(!schema.is_subtype("UnionType", "NotAType"));
    assert!(!schema.is_subtype("NotAType", "Foo"));
    assert!(!schema.is_subtype("Foo", "UnionType"));

    let schema = gen_schema_interfaces("type ObjectType implements Foo & Bar & Baz { me: String }");
    assert!(schema.is_subtype("Foo", "ObjectType"));
    assert!(schema.is_subtype("Bar", "ObjectType"));
    assert!(schema.is_subtype("Baz", "ObjectType"));
    assert!(!schema.is_subtype("Baz", "ObjectType2"));
    assert!(!schema.is_subtype("Foo", "Foo"));
    assert!(!schema.is_subtype("Foo", "Query"));
    assert!(!schema.is_subtype("Foo", "NotAType"));
    assert!(!schema.is_subtype("ObjectType", "Foo"));

    let schema =
        gen_schema_interfaces("interface InterfaceType implements Foo & Bar & Baz { me: String }");
    assert!(schema.is_subtype("Foo", "InterfaceType"));
    assert!(schema.is_subtype("Bar", "InterfaceType"));
    assert!(schema.is_subtype("Baz", "InterfaceType"));
    assert!(!schema.is_subtype("Baz", "InterfaceType2"));
    assert!(!schema.is_subtype("Foo", "Foo"));
    assert!(!schema.is_subtype("Foo", "Query"));
    assert!(!schema.is_subtype("Foo", "NotAType"));
    assert!(!schema.is_subtype("InterfaceType", "Foo"));

    let schema = gen_schema_types("extend union UnionType2 = Baz");
    assert!(schema.is_subtype("UnionType2", "Foo"));
    assert!(schema.is_subtype("UnionType2", "Bar"));
    assert!(schema.is_subtype("UnionType2", "Baz"));

    let schema = gen_schema_interfaces("extend type ObjectType2 implements Baz { me2: String }");
    assert!(schema.is_subtype("Foo", "ObjectType2"));
    assert!(schema.is_subtype("Bar", "ObjectType2"));
    assert!(schema.is_subtype("Baz", "ObjectType2"));

    let schema =
        gen_schema_interfaces("extend interface InterfaceType2 implements Baz { me2: String }");
    assert!(schema.is_subtype("Foo", "InterfaceType2"));
    assert!(schema.is_subtype("Bar", "InterfaceType2"));
    assert!(schema.is_subtype("Baz", "InterfaceType2"));
}

const SUPERGRAPH_BOILERPLATE: &str = r#"
        schema
            @core(feature: "https://specs.apollo.dev/core/v0.1")
            @core(feature: "https://specs.apollo.dev/join/v0.1") {
            query: Query
        }
        directive @core(feature: String!) repeatable on SCHEMA
        directive @join__graph(name: String!, url: String!) on ENUM_VALUE
        enum join__Graph {
            TEST @join__graph(name: "test", url: "http://localhost:4001/graphql")
        }

        "#;

/// https://github.com/graphql/graphql-spec/pull/987
/// https://github.com/apollographql/apollo-rs/issues/682#issuecomment-1752661656
#[test]
fn test_default_root_op_name_ignored_with_explicit_schema_def() {
    let input = r#"
    schema {
        query: Query
        # no mutation here
    }
    type Query {
        viruses: [Virus!]
    }
    type Virus {
        name: String!
        knownMutations: [Mutation!]!
    }
    type Mutation { # happens to use that name but isn't a root operation
        name: String!
        geneSequence: String!
    }
    "#;
    let schema = Schema::parse(input, "schema.graphql");
    schema.validate().unwrap();
    assert!(schema.schema_definition.mutation.is_none())
}
