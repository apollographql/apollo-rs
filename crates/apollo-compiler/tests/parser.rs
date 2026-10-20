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

/// Test inputs write `@u` where a GraphQL `\u` escape sequence belongs, so that the escapes
/// reach the GraphQL lexer verbatim instead of being interpreted as Rust escapes.
fn gql(source: &str) -> String {
    source.replace("@u", "\\u")
}

/// Parse `string_literal` as the description of a directive definition.
fn parse_description(string_literal: &str) -> Result<String, String> {
    let schema = gql(&format!("{string_literal} directive @a on FIELD"));
    match ast::Document::parse(&schema, "schema.graphql") {
        Ok(doc) => {
            let ast::Definition::DirectiveDefinition(def) = &doc.definitions[0] else {
                panic!("expected directive definition")
            };
            Ok(def.description.as_ref().unwrap().to_string())
        }
        Err(err) => Err(err.errors.to_string()),
    }
}

#[test]
fn it_unescapes_september_2025_unicode_escapes() {
    // Variable-width escapes, legacy surrogate pairs, fixed-width escapes, and raw characters.
    assert_eq!(
        parse_description(r#""@u{1F4A9} @uD83D@uDCA9 @u{41} @u0041 💩""#).unwrap(),
        "💩 💩 A A 💩"
    );
    assert_eq!(
        parse_description(r#""@uD800@uDC00 @uDBFF@uDFFF @u{10FFFF}""#).unwrap(),
        "\u{10000} \u{10FFFF} \u{10FFFF}"
    );
}

#[test]
fn it_accepts_control_characters_and_roundtrips_strings() {
    let schema = "type Query { f(a: String = \"x\u{1}\u{7f}\u{85}y@u{1F4A9}\"): Int } # c\u{1}\n";
    let doc = ast::Document::parse(gql(schema), "schema.graphql").unwrap();
    // Serialized form must be re-parseable and stable.
    let printed = doc.to_string();
    let reparsed = ast::Document::parse(&printed, "schema.graphql").unwrap();
    assert_eq!(doc, reparsed);
    assert!(
        printed.contains(&gql("x@u0001\u{7f}\u{85}y💩")),
        "{printed}"
    );
}

/// September 2025 only *expands* the set of accepted string literals: everything that was
/// valid before is still valid, and some previously rejected inputs are now valid.
///
/// The "still valid" and "still invalid" groups pass on the pre-unicode lexer too; the
/// "newly valid" group is what this change adds.
#[test]
fn unicode_support_only_expands_accepted_strings() {
    for still_valid in [
        r#""@u0041""#,
        r#""@u00e9 @u4E2D @uFFFF""#,
        r#""@uD7FF @uE000""#, // just outside the surrogate range
        r#""💩 é 中""#,
        r#""\n\t\\\/\b\f\r\"""#,
    ] {
        assert!(
            parse_description(still_valid).is_ok(),
            "{still_valid} must remain valid"
        );
    }

    for newly_valid in [
        r#""@uD83D@uDCA9""#,
        r#""@uD800@uDC00 @uDBFF@uDFFF""#,
        r#""@u{1F4A9}""#,
        r#""@u{41}""#,
        r#""@u{0000000041}""#,
        r#""@u{10FFFF}""#,
    ] {
        assert!(
            parse_description(newly_valid).is_ok(),
            "{newly_valid} should now be valid"
        );
    }

    // Not a Unicode scalar value, or an unpaired/misordered surrogate.
    for still_invalid in [
        r#""@uD800""#,
        r#""@uDC00""#,
        r#""@uD83D@u0041""#,
        r#""@uDC00@uD800""#,
        r#""@u{D800}""#,
        r#""@u{110000}""#,
        r#""@u{}""#,
        r#""@u{41""#,
    ] {
        assert!(
            parse_description(still_invalid).is_err(),
            "{still_invalid} must stay invalid"
        );
    }
}
