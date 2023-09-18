use super::*;
use crate::ApolloCompiler;
use crate::ReprDatabase;

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
    let expected = r#"directive @customDirective on OBJECT

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
"#;
    let schema = Schema::parse(input);
    assert_eq!(schema.to_string(), expected);
}

#[test]
fn is_subtype() {
    fn gen_schema_types(schema: &str) -> ApolloCompiler {
        let base_schema = with_supergraph_boilerplate(
            r#"
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
            "#,
        );
        let schema = format!("{base_schema}\n{schema}");
        let mut compiler = ApolloCompiler::new();
        compiler.add_document(&schema, "schema.graphql");
        compiler
    }

    fn gen_schema_interfaces(schema: &str) -> ApolloCompiler {
        let base_schema = with_supergraph_boilerplate(
            r#"
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
            "#,
        );
        let schema = format!("{base_schema}\n{schema}");
        let mut compiler = ApolloCompiler::new();
        compiler.add_document(&schema, "schema.graphql");
        compiler
    }

    let ctx = gen_schema_types("union UnionType = Foo | Bar | Baz");
    let schema = ctx.db.schema();
    assert!(schema.is_subtype("UnionType", "Foo"));
    assert!(schema.is_subtype("UnionType", "Bar"));
    assert!(schema.is_subtype("UnionType", "Baz"));
    assert!(!schema.is_subtype("UnionType", "UnionType"));
    assert!(!schema.is_subtype("UnionType", "Query"));
    assert!(!schema.is_subtype("UnionType", "NotAType"));
    assert!(!schema.is_subtype("NotAType", "Foo"));
    assert!(!schema.is_subtype("Foo", "UnionType"));

    let ctx = gen_schema_interfaces("type ObjectType implements Foo & Bar & Baz { me: String }");
    let schema = ctx.db.schema();
    assert!(schema.is_subtype("Foo", "ObjectType"));
    assert!(schema.is_subtype("Bar", "ObjectType"));
    assert!(schema.is_subtype("Baz", "ObjectType"));
    assert!(!schema.is_subtype("Baz", "ObjectType2"));
    assert!(!schema.is_subtype("Foo", "Foo"));
    assert!(!schema.is_subtype("Foo", "Query"));
    assert!(!schema.is_subtype("Foo", "NotAType"));
    assert!(!schema.is_subtype("ObjectType", "Foo"));

    let ctx =
        gen_schema_interfaces("interface InterfaceType implements Foo & Bar & Baz { me: String }");
    let schema = ctx.db.schema();
    assert!(schema.is_subtype("Foo", "InterfaceType"));
    assert!(schema.is_subtype("Bar", "InterfaceType"));
    assert!(schema.is_subtype("Baz", "InterfaceType"));
    assert!(!schema.is_subtype("Baz", "InterfaceType2"));
    assert!(!schema.is_subtype("Foo", "Foo"));
    assert!(!schema.is_subtype("Foo", "Query"));
    assert!(!schema.is_subtype("Foo", "NotAType"));
    assert!(!schema.is_subtype("InterfaceType", "Foo"));

    let ctx = gen_schema_types("extend union UnionType2 = Baz");
    let schema = ctx.db.schema();
    assert!(schema.is_subtype("UnionType2", "Foo"));
    assert!(schema.is_subtype("UnionType2", "Bar"));
    assert!(schema.is_subtype("UnionType2", "Baz"));

    let ctx = gen_schema_interfaces("extend type ObjectType2 implements Baz { me2: String }");
    let schema = ctx.db.schema();
    assert!(schema.is_subtype("Foo", "ObjectType2"));
    assert!(schema.is_subtype("Bar", "ObjectType2"));
    assert!(schema.is_subtype("Baz", "ObjectType2"));

    let ctx =
        gen_schema_interfaces("extend interface InterfaceType2 implements Baz { me2: String }");
    let schema = ctx.db.schema();
    assert!(schema.is_subtype("Foo", "InterfaceType2"));
    assert!(schema.is_subtype("Bar", "InterfaceType2"));
    assert!(schema.is_subtype("Baz", "InterfaceType2"));
}

fn with_supergraph_boilerplate(content: &str) -> String {
    format!(
        "{}\n{}",
        r#"
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

        "#,
        content
    )
}
