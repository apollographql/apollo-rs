use apollo_compiler::ast;
use apollo_compiler::parser::Parser;

#[test]
fn it_errors_when_selection_set_recursion_limit_exceeded() {
    let input = r#"
    query {
      Q1 {
        url {
          hostname
        }
      }
    }
    "#;
    let mut parser = Parser::new().recursion_limit(1);
    let invalid = parser.parse_ast(input, "doc.graphql").unwrap_err();
    assert_eq!(parser.recursion_reached(), 2);
    let errors = invalid.errors.to_string();
    assert!(
        errors.contains("parser recursion limit reached"),
        "{errors}"
    );
    assert_eq!(invalid.partial.definitions.len(), 1);
}

#[test]
fn it_passes_when_selection_set_recursion_limit_is_not_exceeded() {
    let input = r#"
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
    let mut parser = Parser::new().recursion_limit(7);
    let ast = parser.parse_ast(input, "doc.graphql").unwrap();
    assert_eq!(parser.recursion_reached(), 4);
    assert_eq!(ast.definitions.len(), 1);
}

#[test]
fn it_errors_when_selection_set_token_limit_is_exceeded() {
    let schema = r#"
    type Query {
      field(arg1: Int, arg2: Int, arg3: Int, arg4: Int, arg5: Int, arg6: Int): Int
    }
    "#;
    let invalid = Parser::new()
        .token_limit(18)
        .parse_ast(schema, "doc.graphql")
        .unwrap_err();
    let errors = invalid.errors.to_string();
    assert!(
        errors.contains("token limit reached, aborting lexing"),
        "{errors}"
    );
    assert!(errors.contains("doc.graphql:3:30"), "{errors}");
    assert_eq!(invalid.partial.definitions.len(), 1);
}

#[test]
fn it_errors_with_multiple_limits() {
    let input = r#"
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
    let invalid = Parser::new()
        .token_limit(22)
        .recursion_limit(10)
        .parse_ast(input, "doc.graphql")
        .unwrap_err();
    let errors = invalid.errors.to_string();
    assert!(
        errors.contains("token limit reached, aborting lexing"),
        "{errors}"
    );
    assert!(errors.contains("doc.graphql:8:18"), "{errors}");

    let invalid = Parser::new()
        .token_limit(200)
        .recursion_limit(3)
        .parse_ast(input, "doc.graphql")
        .unwrap_err();
    let errors = invalid.errors.to_string();
    assert!(
        errors.contains("parser recursion limit reached"),
        "{errors}"
    );
    assert!(errors.contains("doc.graphql:6:25"), "{errors}");
}

#[test]
fn it_reports_location_for_empty_input() {
    let errors = ast::Document::parse("", "example.graphql")
        .unwrap_err()
        .errors
        .to_string();

    assert!(errors.contains("example.graphql:1:1"), "{errors}");
    assert!(errors.contains("Unexpected <EOF>."), "{errors}");
}
