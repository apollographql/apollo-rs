use apollo_compiler::parser::Parser;
use apollo_compiler::validation::DiagnosticList;
use apollo_compiler::ExecutableDocument;
use apollo_compiler::Schema;

#[test]
fn get_operations() {
    let type_system = r#"
    type Query {
      name: String
    }
    "#;
    let op = r#"{ name }"#;
    let named_op = r#"query getName { name } "#;
    let several_named_op = r#"query getName { name } query getAnotherName { name }"#;
    let empty = r#""#;

    let schema = Schema::parse_and_validate(type_system, "ts.graphql").unwrap();
    let doc = ExecutableDocument::parse_and_validate(&schema, op, "op.graphql").unwrap();
    assert!(doc.operations.get(None).is_ok());

    let doc = ExecutableDocument::parse_and_validate(&schema, named_op, "op.graphql").unwrap();
    assert!(doc.operations.get(Some("getName")).is_ok());
    assert!(doc.operations.get(None).is_ok());

    let doc =
        ExecutableDocument::parse_and_validate(&schema, several_named_op, "op.graphql").unwrap();
    assert!(doc.operations.get(Some("getName")).is_ok());
    assert!(doc.operations.get(None).is_err());

    // Empty document is a syntax error
    let doc = ExecutableDocument::parse_and_validate(&schema, empty, "op.graphql")
        .unwrap_err()
        .partial;
    assert!(doc.operations.get(Some("getName")).is_err());
    assert!(doc.operations.get(None).is_err());
}

#[test]
fn is_introspection_operation() {
    let query_input = r#"
        type Query { foo: String }
        query TypeIntrospect {
          __type(name: "User") {
            name
            fields {
              name
              type {
                name
              }
            }
          }
          __schema {
            types {
              fields {
                name
              }
            }
          }
        }
    "#;
    let (_, doc) = Parser::new()
        .parse_mixed_validate(query_input, "query.graphql")
        .unwrap();
    assert!(doc.operations.named["TypeIntrospect"].is_introspection(&doc));
}

#[test]
fn is_not_introspection_operation() {
    let query_input = r#"
        type Query {
            isKagoshimaWagyuInstock: Boolean
        }

        query CheckStock {
          isKagoshimaWagyuInstock

          __schema {
            types {
              fields {
                name
              }
            }
          }
        }
    "#;
    let mutation_input = r#"
        type Query { unused: Int }
        type Mutation {
            buyA5Wagyu(pounds: Int): String
        }

        mutation PurchaseBasket {
            buyA5Wagyu(pounds: 15)
        }
    "#;

    let (_, query_doc) = Parser::new()
        .parse_mixed_validate(query_input, "query.graphql")
        .unwrap();
    let (_, mutation_doc) = Parser::new()
        .parse_mixed_validate(mutation_input, "mutation.graphql")
        .unwrap();

    assert!(!query_doc.operations.named["CheckStock"].is_introspection(&query_doc));
    assert!(!mutation_doc.operations.named["PurchaseBasket"].is_introspection(&mutation_doc));
}

#[test]
fn is_introspection_deep() {
    let query_input = r#"
      schema {
        query: Root
      }

      type Root {
        species(id: String): Species
      }

      type Species {
        name: String
      }

      query IntrospectDeepFragments {
        ...onRootTrippy
      }

      fragment onRootTrippy on Root {
         ...onRooten
      }

      fragment onRooten on Root {
        ...onRooten2

        ... on Root {
          __schema {
            types {
              name
            }
          }
        }
      }
    "#;
    let introspection_fragment = r#"
      fragment onRooten2 on Root {
        __type(name: "Root") {
          ...onType
        }
        ... on Root {
          __schema {
            directives {
              name
            }
          }
        }
      }

      fragment onType on __Type {
        fields {
          name
        }
      }
    "#;
    let non_introspection_fragment = r#"
      fragment onRooten2 on Root {
        species(id: "Ewok") {
          name
        }

        ... on Root {
          __schema {
            directives {
              name
            }
          }
        }
     }
    "#;

    let query_input_not_introspect = format!("{query_input}{non_introspection_fragment}");
    let query_input = format!("{query_input}{introspection_fragment}");

    let (_, query_doc) = Parser::new()
        .parse_mixed_validate(query_input, "query.graphql")
        .unwrap();
    let (_, query_not_introspect_doc) = Parser::new()
        .parse_mixed_validate(query_input_not_introspect, "query2.graphql")
        .unwrap();

    assert!(query_doc.operations.named["IntrospectDeepFragments"].is_introspection(&query_doc));
    assert!(
        !query_not_introspect_doc.operations.named["IntrospectDeepFragments"]
            .is_introspection(&query_not_introspect_doc)
    );
}

#[test]
fn is_introspection_repeated_fragment() {
    let query_input_indirect = r#"
      type Query { foo: String }

      query IntrospectRepeatedIndirectFragment {
        ...A
        ...B
      }

      fragment A on Query { ...C }
      fragment B on Query { ...C }

      fragment C on Query {
        __schema {
          types {
            name
          }
        }
      }
    "#;

    let query_input_direct = r#"
      type Query { foo: String }

      query IntrospectRepeatedDirectFragment {
        ...C
        ...C
      }

      fragment C on Query {
        __schema {
          types {
            name
          }
        }
      }
    "#;

    let (_, query_doc_indirect) = Parser::new()
        .parse_mixed_validate(query_input_indirect, "indirect.graphql")
        .unwrap();
    let (_, query_doc_direct) = Parser::new()
        .parse_mixed_validate(query_input_direct, "direct.graphql")
        .unwrap();

    assert!(
        query_doc_indirect.operations.named["IntrospectRepeatedIndirectFragment"]
            .is_introspection(&query_doc_indirect)
    );
    assert!(
        query_doc_direct.operations.named["IntrospectRepeatedDirectFragment"]
            .is_introspection(&query_doc_direct)
    );
}

#[test]
fn iter_root_fields() {
    let schema = r#"
        type Query {
          f1: T
          f2: Int
          f3: Int
        }
        type T {
          inner: String
        }
    "#;
    let doc = r#"
      { f1 { inner } ... { f2 } ... F ... F }
      fragment F on Query { f3 }
    "#;
    let schema = Schema::parse_and_validate(schema, "").unwrap();
    let doc = ExecutableDocument::parse_and_validate(&schema, doc, "").unwrap();
    let op = doc.operations.get(None).unwrap();
    assert_eq!(
        op.root_fields(&doc)
            .map(|f| f.name.as_str())
            .collect::<Vec<_>>(),
        ["f1", "f2", "f3"]
    );
}

#[test]
fn iter_all_fields() {
    let schema = r#"
        type Query {
          f1: T
          f2: Int
          f3: Int
        }
        type T {
          inner: String
        }
    "#;
    let doc = r#"
      { f1 { inner } ... { f2 } ... F ... F }
      fragment F on Query { f3 }
    "#;
    let schema = Schema::parse_and_validate(schema, "").unwrap();
    let doc = ExecutableDocument::parse_and_validate(&schema, doc, "").unwrap();
    let op = doc.operations.get(None).unwrap();
    assert_eq!(
        op.all_fields(&doc)
            .map(|f| f.name.as_str())
            .collect::<Vec<_>>(),
        ["f1", "inner", "f2", "f3"]
    );
}

// ExecutableDocumentBuilder tests

#[test]
fn builder_from_multiple_files() {
    let schema_src = r#"
        type Query {
            user: User
            post: Post
        }
        type User { id: ID! name: String }
        type Post { id: ID! title: String }
    "#;
    let schema = Schema::parse_and_validate(schema_src, "schema.graphql").unwrap();

    let query1 = "query GetUser { user { id name } }";
    let query2 = "query GetPost { post { id title } }";

    let mut errors = DiagnosticList::new(Default::default());
    let mut builder = ExecutableDocument::builder(Some(&schema), &mut errors);
    Parser::new().parse_into_executable_builder(query1, "query1.graphql", &mut builder);
    Parser::new().parse_into_executable_builder(query2, "query2.graphql", &mut builder);

    let doc = builder.build();
    assert!(errors.is_empty(), "Expected no errors, got: {}", errors);

    assert_eq!(doc.operations.named.len(), 2);
    assert!(doc.operations.named.contains_key("GetUser"));
    assert!(doc.operations.named.contains_key("GetPost"));
}

#[test]
fn builder_with_fragments_from_multiple_files() {
    let schema_src = r#"
        type Query { user: User }
        type User { id: ID! name: String email: String }
    "#;
    let schema = Schema::parse_and_validate(schema_src, "schema.graphql").unwrap();

    let query = r#"
        query GetUser { user { ...UserFields } }
    "#;
    let fragment = r#"
        fragment UserFields on User { id name email }
    "#;

    let mut errors = DiagnosticList::new(Default::default());
    let mut builder = ExecutableDocument::builder(Some(&schema), &mut errors);
    Parser::new().parse_into_executable_builder(query, "query.graphql", &mut builder);
    Parser::new().parse_into_executable_builder(fragment, "fragment.graphql", &mut builder);

    let doc = builder.build();
    assert!(errors.is_empty(), "Expected no errors, got: {}", errors);

    assert_eq!(doc.operations.named.len(), 1);
    assert_eq!(doc.fragments.len(), 1);
    assert!(doc.operations.named.contains_key("GetUser"));
    assert!(doc.fragments.contains_key("UserFields"));
}

#[test]
fn builder_detects_operation_name_collision() {
    let schema_src = "type Query { field: String }";
    let schema = Schema::parse_and_validate(schema_src, "schema.graphql").unwrap();

    let query1 = "query GetData { field }";
    let query2 = "query GetData { field }";

    let mut errors = DiagnosticList::new(Default::default());
    let mut builder = ExecutableDocument::builder(Some(&schema), &mut errors);
    Parser::new().parse_into_executable_builder(query1, "query1.graphql", &mut builder);
    Parser::new().parse_into_executable_builder(query2, "query2.graphql", &mut builder);

    let _doc = builder.build();
    assert!(
        !errors.is_empty(),
        "Expected errors for operation name collision"
    );

    let error_messages: Vec<String> = errors.iter().map(|e| e.error.to_string()).collect();

    assert!(error_messages
        .iter()
        .any(|msg| msg.contains("GetData") && msg.contains("multiple times")));
}

#[test]
fn builder_detects_fragment_name_collision() {
    let schema_src = "type Query { user: User } type User { id: ID }";
    let schema = Schema::parse_and_validate(schema_src, "schema.graphql").unwrap();

    let fragment1 = "fragment UserData on User { id }";
    let fragment2 = "fragment UserData on User { id }";

    let mut errors = DiagnosticList::new(Default::default());
    let mut builder = ExecutableDocument::builder(Some(&schema), &mut errors);
    Parser::new().parse_into_executable_builder(fragment1, "fragment1.graphql", &mut builder);
    Parser::new().parse_into_executable_builder(fragment2, "fragment2.graphql", &mut builder);

    let _doc = builder.build();
    assert!(
        !errors.is_empty(),
        "Expected errors for fragment name collision"
    );

    let error_messages: Vec<String> = errors.iter().map(|e| e.error.to_string()).collect();

    assert!(error_messages
        .iter()
        .any(|msg| msg.contains("UserData") && msg.contains("multiple times")));
}

#[test]
fn builder_without_schema() {
    let query1 = "query GetData { field }";
    let query2 = "query GetMore { other }";

    let mut errors = DiagnosticList::new(Default::default());
    let mut builder = ExecutableDocument::builder(None, &mut errors);
    Parser::new().parse_into_executable_builder(query1, "query1.graphql", &mut builder);
    Parser::new().parse_into_executable_builder(query2, "query2.graphql", &mut builder);

    let doc = builder.build();
    assert!(errors.is_empty(), "Expected no errors, got: {}", errors);

    assert_eq!(doc.operations.named.len(), 2);
    assert!(doc.operations.named.contains_key("GetData"));
    assert!(doc.operations.named.contains_key("GetMore"));
}

#[test]
fn builder_preserves_source_information() {
    let schema_src = "type Query { field: String }";
    let schema = Schema::parse_and_validate(schema_src, "schema.graphql").unwrap();

    let query1 = "query Q1 { field }";
    let query2 = "query Q2 { field }";

    let mut errors = DiagnosticList::new(Default::default());
    let mut builder = ExecutableDocument::builder(Some(&schema), &mut errors);
    Parser::new().parse_into_executable_builder(query1, "query1.graphql", &mut builder);
    Parser::new().parse_into_executable_builder(query2, "query2.graphql", &mut builder);

    let doc = builder.build();
    assert!(errors.is_empty(), "Expected no errors, got: {}", errors);

    // Verify that source information is tracked
    assert_eq!(doc.sources.len(), 2);
    assert_eq!(doc.sources[0].path(), "query1.graphql");
    assert_eq!(doc.sources[1].path(), "query2.graphql");
}

#[test]
fn builder_handles_anonymous_and_named_operations() {
    let schema_src = "type Query { field: String }";
    let schema = Schema::parse_and_validate(schema_src, "schema.graphql").unwrap();

    let anonymous = "{ field }";
    let named = "query GetData { field }";

    let mut errors = DiagnosticList::new(Default::default());
    let mut builder = ExecutableDocument::builder(Some(&schema), &mut errors);
    Parser::new().parse_into_executable_builder(anonymous, "anonymous.graphql", &mut builder);
    Parser::new().parse_into_executable_builder(named, "named.graphql", &mut builder);

    let _doc = builder.build();
    // Should error because mixing anonymous and named operations is ambiguous
    assert!(
        !errors.is_empty(),
        "Expected errors for mixing anonymous and named operations"
    );
}

#[test]
fn builder_with_multiple_fragments_used_in_query() {
    let schema_src = r#"
        type Query { user: User }
        type User {
            id: ID!
            profile: Profile
            settings: Settings
        }
        type Profile { name: String bio: String }
        type Settings { theme: String notifications: Boolean }
    "#;
    let schema = Schema::parse_and_validate(schema_src, "schema.graphql").unwrap();

    let query = r#"
        query GetUser {
            user {
                id
                ...ProfileFields
                ...SettingsFields
            }
        }
    "#;
    let profile_fragment = "fragment ProfileFields on Profile { name bio }";
    let settings_fragment = "fragment SettingsFields on Settings { theme notifications }";

    let mut errors = DiagnosticList::new(Default::default());
    let mut builder = ExecutableDocument::builder(Some(&schema), &mut errors);
    Parser::new().parse_into_executable_builder(query, "query.graphql", &mut builder);
    Parser::new().parse_into_executable_builder(profile_fragment, "profile.graphql", &mut builder);
    Parser::new().parse_into_executable_builder(
        settings_fragment,
        "settings.graphql",
        &mut builder,
    );

    let doc = builder.build();
    assert!(errors.is_empty(), "Expected no errors, got: {}", errors);

    assert_eq!(doc.operations.named.len(), 1);
    assert_eq!(doc.fragments.len(), 2);
    assert!(doc.fragments.contains_key("ProfileFields"));
    assert!(doc.fragments.contains_key("SettingsFields"));
}
