use apollo_compiler::execution::SchemaIntrospectionSplit;
use apollo_compiler::ExecutableDocument;
use apollo_compiler::Schema;
use expect_test::expect;

const SCHEMA: &str = r#"
    schema {
        query: Fibonacci
        mutation: Fibonacci
    }

    type Fibonacci {
        value: Int
        next: Fibonacci
    }
"#;

#[test]
fn test_none() {
    let doc = r#"
        query TheOperation { value }
        query OtherOperation { __schema { description } }
    "#;
    let expected = expect!["None"];
    assert_split(doc, expected);
}

#[test]
fn test_only() {
    let doc = r#"
        query TheOperation { __schema { description } }
        query OtherOperation { value }
    "#;
    let expected = expect![[r#"
        Only:
        query TheOperation {
          __schema {
            description
          }
        }
    "#]];
    assert_split(doc, expected);
}

#[test]
fn test_both() {
    let doc = r#"
        query TheOperation { value __schema { description } }
    "#;
    let expected = expect![[r#"
        Introspection parts:
        query TheOperation {
          __schema {
            description
          }
        }

        Other parts:
        query TheOperation {
          value
        }
    "#]];
    assert_split(doc, expected);
}

#[test]
fn test_nested() {
    let doc = r#"
        query TheOperation { value next { __schema { description } } }
    "#;
    let expected = expect![[r#"
        GraphQLError {
            message: "Schema introspection field __schema is only supported at the root of a query",
            locations: [
                GraphQLLocation {
                    line: 2,
                    column: 43,
                },
            ],
            path: [],
            extensions: {},
        }
    "#]];
    assert_split(doc, expected);
}

#[test]
fn test_mutation() {
    let doc = r#"
        mutation TheOperation { __schema { description } }
    "#;
    let expected = expect![[r#"
        GraphQLError {
            message: "Schema introspection field __schema is only supported at the root of a query",
            locations: [
                GraphQLLocation {
                    line: 2,
                    column: 33,
                },
            ],
            path: [],
            extensions: {},
        }
    "#]];
    assert_split(doc, expected);
}

#[test]
fn test_mutation_nested() {
    let doc = r#"
        mutation TheOperation { value next { __schema { description } } }
    "#;
    let expected = expect![[r#"
        GraphQLError {
            message: "Schema introspection field __schema is only supported at the root of a query",
            locations: [
                GraphQLLocation {
                    line: 2,
                    column: 46,
                },
            ],
            path: [],
            extensions: {},
        }
    "#]];
    assert_split(doc, expected);
}

#[track_caller]
fn assert_split(doc: &str, expected: expect_test::Expect) {
    let schema = Schema::parse_and_validate(SCHEMA, "schema.graphql").unwrap();
    let doc = ExecutableDocument::parse_and_validate(&schema, doc, "doc.graphql").unwrap();
    let operation = doc.get_operation(Some("TheOperation")).unwrap();

    match SchemaIntrospectionSplit::split(&schema, &doc, operation) {
        Ok(SchemaIntrospectionSplit::None) => expected.assert_eq("None"),
        Ok(SchemaIntrospectionSplit::Only(introspection_query)) => {
            expected.assert_eq(&format!("Only:\n{introspection_query}"))
        }
        Ok(SchemaIntrospectionSplit::Both {
            introspection_query,
            filtered_document,
        }) => expected.assert_eq(&format!(
            "Introspection parts:\n{introspection_query}\nOther parts:\n{filtered_document}"
        )),
        Err(err) => expected.assert_debug_eq(&err.into_graphql_error(&doc.sources)),
    }
}
