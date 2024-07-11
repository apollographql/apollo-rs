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
        next(n: Int = 1): Fibonacci
    }

    input In {
        list: [Int]
    }

    directive @someDir(arg: In) on FRAGMENT_DEFINITION
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
            message: "Schema introspection field __schema is not supported nested in other fields",
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
fn test_nested_in_fragment() {
    let doc = r#"
        query TheOperation { ...A next { ...A } }
        fragment A on Fibonacci { value __schema { description } }
    "#;
    let expected = expect![[r#"
        GraphQLError {
            message: "Schema introspection field __schema is not supported nested in other fields",
            locations: [
                GraphQLLocation {
                    line: 3,
                    column: 41,
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
            message: "Schema introspection field __schema is not supported in a mutation operation",
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
            message: "Schema introspection field __schema is not supported in a mutation operation",
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

#[test]
fn test_nested_in_other_operation() {
    let doc = r#"
        query TheOperation { value }
        query OtherOperation { value next { __schema { description } } }
    "#;
    let expected = expect!["None"];
    assert_split(doc, expected);
}

#[test]
fn test_none_with_fragments() {
    let doc = r#"
        query TheOperation { ... { ...A } }
        query OtherOperation { ... { ...C } }
        fragment A on Fibonacci { ...B }
        fragment B on Fibonacci { ... { value } }
        fragment C on Fibonacci { ...D }
        fragment D on Fibonacci { ... { __schema { description } } }
    "#;
    let expected = expect!["None"];
    assert_split(doc, expected);
}

#[test]
fn test_only_with_fragments() {
    let doc = r#"
        query TheOperation { ... { ...A } }
        query OtherOperation { ... { ...C } }
        fragment A on Fibonacci { ...B }
        fragment B on Fibonacci { ... { __schema { description } } }
        fragment C on Fibonacci { ...D }
        fragment D on Fibonacci { ... { value } }
    "#;
    let expected = expect![[r#"
        Only:
        query TheOperation {
          ... {
            ...A
          }
        }

        fragment A on Fibonacci {
          ...B
        }

        fragment B on Fibonacci {
          ... {
            __schema {
              description
            }
          }
        }
    "#]];
    assert_split(doc, expected);
}

#[test]
fn test_both_with_fragments() {
    let doc = r#"
        query TheOperation { ... { ...A } }
        query OtherOperation { ...O }
        fragment A on Fibonacci { ...B ...D }
        fragment B on Fibonacci { value next { ... { ...C } } }
        fragment C on Fibonacci { value }
        fragment D on Fibonacci { __schema { ...S } }
        fragment O on Fibonacci { __typename }
        fragment S on __Schema { description }
    "#;
    let expected = expect![[r#"
        Introspection parts:
        query TheOperation {
          ... {
            ...A
          }
        }

        fragment S on __Schema {
          description
        }

        fragment D on Fibonacci {
          __schema {
            ...S
          }
        }

        fragment A on Fibonacci {
          ...D
        }

        Other parts:
        query TheOperation {
          ... {
            ...A
          }
        }

        fragment C on Fibonacci {
          value
        }

        fragment B on Fibonacci {
          value
          next {
            ... {
              ...C
            }
          }
        }

        fragment A on Fibonacci {
          ...B
        }
    "#]];
    assert_split(doc, expected);
}

#[test]
fn test_variables() {
    let doc = r#"
        query TheOperation(
            $v1: Boolean!,
            $v2: Boolean!,
            $v3: Int!,
            $v4: Boolean!,
            $v5: Int!,
            $v6: Boolean!,
            $v7: Boolean!,
            $v8: Int!,
            $v9: Int!,
            $v10: Boolean!,
            $v11: Int!,
            $v12: Boolean!,
            $v13: Boolean!,
        ) {
            ... A @skip(if: $v1)
            ... @skip(if: $v2) {
                next(n: $v3) { ...B }
            }
            ... @skip(if: $v4) { ...A }
        }
        fragment A on Fibonacci @someDir(arg: {list: [$v5]}) {
            value @skip(if: $v6)
            __schema {
                ...C @skip(if: $v7)
            }
        }
        fragment B on Fibonacci @someDir(arg: {list: [$v8]}) {
            next(n: $v9) {
                value @skip(if: $v10)
            }
        }
        fragment C on __Schema @someDir(arg: {list: [$v11]}) {
            ... @skip(if: $v12) {
                description @skip(if: $v13)
            }
        }
    "#;
    let expected = expect![[r#"
        Introspection parts:
        query TheOperation($v1: Boolean!, $v4: Boolean!, $v5: Int!, $v7: Boolean!, $v11: Int!, $v12: Boolean!, $v13: Boolean!) {
          ...A @skip(if: $v1)
          ... @skip(if: $v4) {
            ...A
          }
        }

        fragment C on __Schema @someDir(arg: {list: [$v11]}) {
          ... @skip(if: $v12) {
            description @skip(if: $v13)
          }
        }

        fragment A on Fibonacci @someDir(arg: {list: [$v5]}) {
          __schema {
            ...C @skip(if: $v7)
          }
        }

        Other parts:
        query TheOperation($v1: Boolean!, $v2: Boolean!, $v3: Int!, $v4: Boolean!, $v5: Int!, $v6: Boolean!, $v8: Int!, $v9: Int!, $v10: Boolean!) {
          ...A @skip(if: $v1)
          ... @skip(if: $v2) {
            next(n: $v3) {
              ...B
            }
          }
          ... @skip(if: $v4) {
            ...A
          }
        }

        fragment A on Fibonacci @someDir(arg: {list: [$v5]}) {
          value @skip(if: $v6)
        }

        fragment B on Fibonacci @someDir(arg: {list: [$v8]}) {
          next(n: $v9) {
            value @skip(if: $v10)
          }
        }
    "#]];
    assert_split(doc, expected);
}

#[track_caller]
fn assert_split(doc: &str, expected: expect_test::Expect) {
    let schema = Schema::parse_and_validate(SCHEMA, "schema.graphql").unwrap();
    let doc = ExecutableDocument::parse_and_validate(&schema, doc, "doc.graphql").unwrap();
    let operation = doc.operations.get(Some("TheOperation")).unwrap();

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
