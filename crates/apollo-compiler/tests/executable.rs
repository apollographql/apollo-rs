use apollo_compiler::ApolloCompiler;
use apollo_compiler::HirDatabase;
use apollo_compiler::ReprDatabase;

#[test]
fn find_operations() {
    let type_system = r#"
type Query {
name: String
}
    "#;
    let op = r#"{ name }"#;
    let named_op = r#"query getName { name } "#;
    let several_named_op = r#"query getName { name } query getAnotherName { name }"#;
    let noop = r#""#;

    let mut compiler = ApolloCompiler::new();
    compiler.add_type_system(type_system, "ts.graphql");
    let op_id = compiler.add_executable(op, "op.graphql");
    let op = compiler.db.find_operation(op_id, None);
    assert!(op.is_some());

    compiler.update_executable(op_id, named_op);
    let op = compiler.db.find_operation(op_id, Some("getName".into()));
    assert!(op.is_some());
    let op = compiler.db.find_operation(op_id, None);
    assert!(op.is_some());

    compiler.update_executable(op_id, several_named_op);
    let op = compiler.db.find_operation(op_id, Some("getName".into()));
    assert!(op.is_some());
    let op = compiler.db.find_operation(op_id, None);
    assert!(op.is_none());

    compiler.update_executable(op_id, noop);
    let op = compiler.db.find_operation(op_id, Some("getName".into()));
    assert!(op.is_none());

    let op = compiler.db.find_operation(op_id, None);
    assert!(op.is_none());
}

#[test]
fn is_introspection_operation() {
    let query_input = r#"
        type Query {}
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

    let mut compiler = ApolloCompiler::new();
    let id = compiler.add_document(query_input, "query.graphql");
    let doc = compiler.db.executable_document(id);
    assert!(doc.named_operations["TypeIntrospect"].is_introspection(&doc));
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
        type Mutation {
            buyA5Wagyu(pounds: Int): String
        }

        mutation PurchaseBasket {
            buyA5Wagyu(pounds: 15)
        }
    "#;

    let mut compiler = ApolloCompiler::new();
    let query_id = compiler.add_document(query_input, "query.graphql");
    let mutation_id = compiler.add_document(mutation_input, "mutation.graphql");

    let query_doc = compiler.db.executable_document(query_id);
    let mutation_doc = compiler.db.executable_document(mutation_id);

    assert!(!query_doc.named_operations["CheckStock"].is_introspection(&query_doc));
    assert!(!mutation_doc.named_operations["PurchaseBasket"].is_introspection(&mutation_doc));
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

      fragment onRooten2_not_intro on Root {
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

    let query_input_not_introspect = query_input.replace("...onRooten2", "...onRooten2_not_intro");

    let mut compiler = ApolloCompiler::new();
    let query_id = compiler.add_document(query_input, "query.graphql");
    let query_not_introspect_id =
        compiler.add_document(&query_input_not_introspect, "query2.graphql");

    let query_doc = compiler.db.executable_document(query_id);
    let query_not_introspect_doc = compiler.db.executable_document(query_not_introspect_id);

    assert!(query_doc.named_operations["IntrospectDeepFragments"].is_introspection(&query_doc));
    assert!(
        !query_not_introspect_doc.named_operations["IntrospectDeepFragments"]
            .is_introspection(&query_not_introspect_doc)
    );
}

#[test]
fn is_introspection_repeated_fragment() {
    let query_input_indirect = r#"
      type Query {}

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
      type Query {}

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

    let mut compiler = ApolloCompiler::new();
    let query_id_indirect = compiler.add_document(query_input_indirect, "indirect.graphql");
    let query_id_direct = compiler.add_document(query_input_direct, "direct.graphql");

    let query_doc_indirect = compiler.db.executable_document(query_id_indirect);
    let query_doc_direct = compiler.db.executable_document(query_id_direct);

    assert!(
        query_doc_indirect.named_operations["IntrospectRepeatedIndirectFragment"]
            .is_introspection(&query_doc_indirect)
    );
    assert!(
        query_doc_direct.named_operations["IntrospectRepeatedDirectFragment"]
            .is_introspection(&query_doc_direct)
    );
}
