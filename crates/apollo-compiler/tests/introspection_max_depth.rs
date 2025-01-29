use apollo_compiler::introspection;
use apollo_compiler::ExecutableDocument;
use apollo_compiler::Schema;
use expect_test::expect;

#[test]
fn test_3_sibling_fields_list() {
    let doc = r#"
      {
        __type(name: "Query") {
          trueFields: fields(includeDeprecated: true) {
            name
          }
          falseFields: fields(includeDeprecated: false) {
            name
          }
          omittedFields: fields {
            name
          }
        }
      }
    "#;
    let expected = expect!["Ok"];
    assert_split(doc, expected);
}

#[test]
fn test_2_nested_fields_lists() {
    let doc = r#"
      {
        __type(name: "Query") {
          fields {
            type {
              fields {
                name
              }
            }
          }
        }
      }
    "#;
    let expected = expect!["Ok"];
    assert_split(doc, expected);
}

#[test]
fn test_3_nested_fields_lists() {
    let doc = r#"
      {
        __type(name: "Query") {
          fields {
            type {
              fields {
                type {
                  fields {
                    name
                  }
                }
              }
            }
          }
        }
      }
    "#;
    let expected = expect![[r#"
        GraphQLError {
            message: "Maximum introspection depth exceeded",
            locations: [
                8:19,
            ],
            path: [],
            extensions: {},
        }
    "#]];
    assert_split(doc, expected);
}

#[test]
fn test_2_nested_input_fields_lists() {
    let doc = r#"
      {
        __type(name: "Query") {
          inputFields {
            type {
              inputFields {
                name
              }
            }
          }
        }
      }
    "#;
    let expected = expect!["Ok"];
    assert_split(doc, expected);
}

#[test]
fn test_3_nested_input_fields_lists() {
    let doc = r#"
      {
        __type(name: "Query") {
          inputFields {
            type {
              inputFields {
                type {
                  inputFields {
                    name
                  }
                }
              }
            }
          }
        }
      }
    "#;
    let expected = expect![[r#"
        GraphQLError {
            message: "Maximum introspection depth exceeded",
            locations: [
                8:19,
            ],
            path: [],
            extensions: {},
        }
    "#]];
    assert_split(doc, expected);
}

#[test]
fn test_2_nested_interfaces_lists() {
    let doc = r#"
      {
        __schema {
          types {
            interfaces {
              interfaces {
                name
              }
            }
          }
        }
      }
    "#;
    let expected = expect!["Ok"];
    assert_split(doc, expected);
}

#[test]
fn test_3_nested_interfaces_lists() {
    let doc = r#"
      {
        __schema {
          types {
            interfaces {
              interfaces {
                interfaces {
                  name
                }
              }
            }
          }
        }
      }
    "#;
    let expected = expect![[r#"
        GraphQLError {
            message: "Maximum introspection depth exceeded",
            locations: [
                7:17,
            ],
            path: [],
            extensions: {},
        }
    "#]];
    assert_split(doc, expected);
}

#[test]
fn test_2_nested_possible_types_lists() {
    let doc = r#"
      {
        __schema {
          types {
            possibleTypes {
              possibleTypes {
                name
              }
            }
          }
        }
      }
    "#;
    let expected = expect!["Ok"];
    assert_split(doc, expected);
}

#[test]
fn test_3_nested_possible_types_lists() {
    let doc = r#"
      {
        __schema {
          types {
            possibleTypes {
              possibleTypes {
                possibleTypes {
                  name
                }
              }
            }
          }
        }
      }
    "#;
    let expected = expect![[r#"
        GraphQLError {
            message: "Maximum introspection depth exceeded",
            locations: [
                7:17,
            ],
            path: [],
            extensions: {},
        }
    "#]];
    assert_split(doc, expected);
}

#[test]
fn test_2_nested_possible_types_lists_with_inline_fragments() {
    let doc = r#"
      {
        ... on Query {
          __schema {
            types {
              ... on __Type {
                possibleTypes {
                  ... on __Type {
                    possibleTypes {
                      ... on __Type {
                        name
                      }
                    }
                  }
                }
              }
            }
          }
        }
      }
    "#;
    let expected = expect!["Ok"];
    assert_split(doc, expected);
}

#[test]
fn test_3_nested_possible_types_lists_with_inline_fragments() {
    let doc = r#"
      {
        ... on Query {
          __schema {
            types {
              ... on __Type {
                possibleTypes {
                  ... on __Type {
                    possibleTypes {
                      ... on __Type {
                        possibleTypes {
                          ... on __Type {
                            name
                          }
                        }
                      }
                    }
                  }
                }
              }
            }
          }
        }
      }
    "#;
    let expected = expect![[r#"
        GraphQLError {
            message: "Maximum introspection depth exceeded",
            locations: [
                11:25,
            ],
            path: [],
            extensions: {},
        }
    "#]];
    assert_split(doc, expected);
}

#[test]
fn test_2_nested_possible_types_lists_with_named_fragments() {
    let doc = r#"
      {
        __schema {
          types {
            ...One
          }
        }
      }
      fragment One on __Type {
        possibleTypes {
          ...Two
        }
      }
      fragment Two on __Type {
        possibleTypes {
          ...Three
        }
      }
      fragment Three on __Type {
        name
      }
    "#;
    let expected = expect!["Ok"];
    assert_split(doc, expected);
}

#[test]
fn test_3_nested_possible_types_lists_with_named_fragments() {
    let doc = r#"
      {
        __schema {
          types {
            ...One
          }
        }
      }
      fragment One on __Type {
        possibleTypes {
          ...Two
        }
      }
      fragment Two on __Type {
        possibleTypes {
          ...Three
        }
      }
      fragment Three on __Type {
        possibleTypes {
          ...Four
        }
      }
      fragment Four on __Type {
        name
      }
    "#;
    let expected = expect![[r#"
        GraphQLError {
            message: "Maximum introspection depth exceeded",
            locations: [
                20:9,
            ],
            path: [],
            extensions: {},
        }
    "#]];
    assert_split(doc, expected);
}

#[track_caller]
fn assert_split(doc: &str, expected: expect_test::Expect) {
    let schema = "type Query { f: Int }";
    let schema = Schema::parse_and_validate(schema, "schema.graphql").unwrap();
    let doc = ExecutableDocument::parse_and_validate(&schema, doc, "doc.graphql").unwrap();
    let operation = doc.operations.get(None).unwrap();

    match introspection::check_max_depth(&doc, operation) {
        Ok(_) => expected.assert_eq("Ok"),
        Err(err) => expected.assert_debug_eq(&err.to_graphql_error(&doc.sources)),
    }
}
