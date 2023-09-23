use apollo_compiler::ApolloCompiler;
use apollo_compiler::CstDatabase;
use apollo_compiler::HirDatabase;

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
    let doc_id = compiler.add_document(schema, "schema.graphql");

    let ast = compiler.db.cst(doc_id);

    assert_eq!(ast.recursion_limit().high, 2);
    assert_eq!(ast.errors().len(), 1);
    assert_eq!(ast.document().definitions().count(), 1);
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
    let doc_id = compiler.add_document(schema, "schema.graphql");

    let ast = compiler.db.cst(doc_id);

    assert_eq!(ast.recursion_limit().high, 4);
    assert_eq!(ast.errors().len(), 0);
    assert_eq!(ast.document().definitions().count(), 1);
}

#[test]
fn it_errors_when_selection_set_token_limit_is_exceeded() {
    let schema = r#"
    type Query {
      field(arg1: Int, arg2: Int, arg3: Int, arg4: Int, arg5: Int, arg6: Int): Int
    }
    "#;
    let mut compiler = ApolloCompiler::new().token_limit(18);
    let doc_id = compiler.add_document(schema, "schema.graphql");

    let ast = compiler.db.cst(doc_id);

    assert_eq!(ast.errors().len(), 1);
    assert_eq!(
        ast.errors().next(),
        Some(&apollo_parser::Error::limit(
            "token limit reached, aborting lexing",
            47
        ))
    );
    assert_eq!(ast.document().definitions().count(), 1);
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
    let doc_id = compiler.add_document(schema, "schema.graphql");

    let ast = compiler.db.cst(doc_id);

    assert_eq!(ast.errors().len(), 1);
    assert_eq!(
        ast.errors().next(),
        Some(&apollo_parser::Error::limit(
            "token limit reached, aborting lexing",
            142
        ))
    );

    let mut compiler = ApolloCompiler::new().recursion_limit(3).token_limit(200);
    let doc_id = compiler.add_document(schema, "schema.graphql");

    let ast = compiler.db.cst(doc_id);

    assert_eq!(ast.errors().len(), 1);
    assert_eq!(
        ast.errors().next(),
        Some(&apollo_parser::Error::limit(
            "parser recursion limit reached",
            101
        ))
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
    let ts = compiler.db.type_system();

    let mut compiler2 = ApolloCompiler::new().token_limit(2);
    compiler2.set_type_system_hir(ts);
    compiler2.add_executable(query, "query.graphql");
    let parser_errors = compiler2.db.syntax_errors();

    assert_eq!(parser_errors.len(), 1);
}
