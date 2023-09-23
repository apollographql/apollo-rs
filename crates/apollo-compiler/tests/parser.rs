use apollo_compiler::ApolloCompiler;
use apollo_compiler::ReprDatabase;

#[test]
fn it_errors_when_selection_set_recursion_limit_exceeded() {
    let schema = r#"
    query {
      Q1 {
        url {
          hostname
        }
      }
    }
    "#;
    let mut compiler = ApolloCompiler::new().recursion_limit(1);
    let id = compiler.add_document(schema, "schema.graphql");
    let ast = compiler.db.ast(id);
    assert_eq!(compiler.db.recursion_reached(id), 2);
    assert_eq!(ast.parse_errors().len(), 1);
    assert_eq!(ast.definitions.len(), 1);
}

#[test]
fn it_passes_when_selection_set_recursion_limit_is_not_exceeded() {
    let schema = r#"
    query {
      Q1 {
        Q2 {
          Q3 {
            url
          }
        }
      }
    }
    "#;
    let mut compiler = ApolloCompiler::new().recursion_limit(7);
    let id = compiler.add_document(schema, "schema.graphql");
    let ast = compiler.db.ast(id);
    assert_eq!(compiler.db.recursion_reached(id), 4);
    assert_eq!(ast.parse_errors().len(), 0);
    assert_eq!(ast.definitions.len(), 1);
}

#[test]
fn it_errors_when_selection_set_token_limit_is_exceeded() {
    let schema = r#"
    type Query {
      field(arg1: Int, arg2: Int, arg3: Int, arg4: Int, arg5: Int, arg6: Int): Int
    }
    "#;
    let mut compiler = ApolloCompiler::new().token_limit(18);
    let id = compiler.add_document(schema, "schema.graphql");
    let ast = compiler.db.ast(id);
    assert_eq!(ast.parse_errors().len(), 1);
    assert_eq!(
        format!("{:?}", ast.parse_errors()[0]),
        "ERROR@47:47 \"token limit reached, aborting lexing\" "
    );
    assert_eq!(ast.definitions.len(), 1);
}

#[test]
fn it_errors_with_multiple_limits() {
    let schema = r#"
        query {
            a {
                a {
                    a {
                        a
                    }
                }
            }
        }
    "#;
    let mut compiler = ApolloCompiler::new().token_limit(22).recursion_limit(10);
    let id = compiler.add_document(schema, "schema.graphql");
    let ast = compiler.db.ast(id);
    assert_eq!(ast.parse_errors().len(), 1);
    assert_eq!(
        format!("{:?}", ast.parse_errors()[0]),
        "ERROR@142:142 \"token limit reached, aborting lexing\" "
    );

    let mut compiler = ApolloCompiler::new().recursion_limit(3).token_limit(200);
    let id = compiler.add_document(schema, "schema.graphql");
    let ast = compiler.db.ast(id);
    assert_eq!(ast.parse_errors().len(), 1);
    assert_eq!(
        format!("{:?}", ast.parse_errors()[0]),
        "ERROR@101:101 \"parser recursion limit reached\" "
    );
}

#[test]
fn token_limit_with_multiple_sources() {
    let schema = r#"
    type Query {
        website: URL,
        amount: Int
    }

    scalar URL @specifiedBy(url: "a.com");
    "#;
    let query = "{ website }";

    let mut compiler = ApolloCompiler::new();
    compiler.add_type_system(schema, "schema.graphql");
    let schema = compiler.db.schema();

    let mut compiler2 = ApolloCompiler::from_schema(schema).token_limit(2);
    let id = compiler2.add_executable(query, "query.graphql");
    let ast = compiler2.db.ast(id);
    assert_eq!(ast.parse_errors().len(), 1);
}
