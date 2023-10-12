use apollo_compiler::Parser;

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
    let ast = parser.parse_ast(input, "doc.graphql");
    assert_eq!(parser.recursion_reached(), 2);
    let errors = ast.check_parse_errors().unwrap_err().to_string_no_color();
    assert!(
        errors.contains("parser recursion limit reached"),
        "{errors}"
    );
    assert_eq!(ast.definitions.len(), 1);
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
    let ast = parser.parse_ast(input, "doc.graphql");
    assert_eq!(parser.recursion_reached(), 4);
    ast.check_parse_errors().unwrap();
    assert_eq!(ast.definitions.len(), 1);
}

#[test]
fn it_errors_when_selection_set_token_limit_is_exceeded() {
    let schema = r#"
    type Query {
      field(arg1: Int, arg2: Int, arg3: Int, arg4: Int, arg5: Int, arg6: Int): Int
    }
    "#;
    let ast = Parser::new()
        .token_limit(18)
        .parse_ast(schema, "doc.graphql");
    let errors = ast.check_parse_errors().unwrap_err().to_string_no_color();
    assert!(
        errors.contains("token limit reached, aborting lexing"),
        "{errors}"
    );
    assert!(errors.contains("doc.graphql:3:30"), "{errors}");
    assert_eq!(ast.definitions.len(), 1);
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
    let ast = Parser::new()
        .token_limit(22)
        .recursion_limit(10)
        .parse_ast(input, "doc.graphql");
    let errors = ast.check_parse_errors().unwrap_err().to_string_no_color();
    assert!(
        errors.contains("token limit reached, aborting lexing"),
        "{errors}"
    );
    assert!(errors.contains("doc.graphql:8:18"), "{errors}");

    let ast = Parser::new()
        .token_limit(200)
        .recursion_limit(3)
        .parse_ast(input, "doc.graphql");
    let errors = ast.check_parse_errors().unwrap_err().to_string_no_color();
    assert!(
        errors.contains("parser recursion limit reached"),
        "{errors}"
    );
    assert!(errors.contains("doc.graphql:6:25"), "{errors}");
}
