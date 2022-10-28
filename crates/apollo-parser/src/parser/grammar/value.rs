use crate::{
    parser::grammar::{name, variable},
    Parser, SyntaxKind, TokenKind, S, T,
};

/// See: https://spec.graphql.org/October2021/#Value
///
/// *Value*
///     Variable
///     IntValue
///     FloatValue
///     StringValue
///     BooleanValue
///     NullValue
///     EnumValue
///     ListValue
///     ObjectValue
pub(crate) fn value(p: &mut Parser, pop_on_error: bool) {
    match p.peek() {
        Some(T![$]) => variable::variable(p),
        Some(TokenKind::Int) => {
            let _g = p.start_node(SyntaxKind::INT_VALUE);
            p.bump(SyntaxKind::INT);
        }
        Some(TokenKind::Float) => {
            let _g = p.start_node(SyntaxKind::FLOAT_VALUE);
            p.bump(SyntaxKind::FLOAT);
        }
        Some(TokenKind::StringValue) => {
            let _g = p.start_node(SyntaxKind::STRING_VALUE);
            p.bump(SyntaxKind::STRING);
        }
        Some(TokenKind::Name) => {
            let node = p.peek_data().unwrap();
            match node.as_str() {
                "true" => {
                    let _g = p.start_node(SyntaxKind::BOOLEAN_VALUE);
                    p.bump(SyntaxKind::true_KW);
                }
                "false" => {
                    let _g = p.start_node(SyntaxKind::BOOLEAN_VALUE);
                    p.bump(SyntaxKind::false_KW);
                }
                "null" => {
                    let _g = p.start_node(SyntaxKind::NULL_VALUE);
                    p.bump(SyntaxKind::null_KW)
                }
                _ => enum_value(p),
            }
        }
        Some(T!['[']) => list_value(p),
        Some(T!['{']) => object_value(p),
        _ => {
            let error_message = "expected a valid Value";
            if pop_on_error {
                p.err_and_pop(error_message);
            } else {
                p.err(error_message);
            }
        }
    }
}
/// See: https://spec.graphql.org/October2021/#EnumValue
///
/// *EnumValue*:
///     Name *but not* **true** *or* **false** *or* **null**
pub(crate) fn enum_value(p: &mut Parser) {
    let _g = p.start_node(SyntaxKind::ENUM_VALUE);
    let name = p.peek_data().unwrap();

    if matches!(name.as_str(), "true" | "false" | "null") {
        p.err("unexpected Enum Value");
    }

    name::name(p);
}

/// See: https://spec.graphql.org/October2021/#ListValue
///
/// *ListValue*:
///     **[** **]**
///     **[** Value* **]**
pub(crate) fn list_value(p: &mut Parser) {
    let _g = p.start_node(SyntaxKind::LIST_VALUE);
    p.bump(S!['[']);

    while let Some(node) = p.peek() {
        if node == T![']'] {
            p.bump(S![']']);
            break;
        } else if node == TokenKind::Eof {
            break;
        } else {
            value(p, true);
        }
    }
}

/// See: https://spec.graphql.org/October2021/#ObjectValue
///
/// *ObjectValue*:
///     **{** **}**
///     **{** ObjectField* **}**
pub(crate) fn object_value(p: &mut Parser) {
    let _g = p.start_node(SyntaxKind::OBJECT_VALUE);
    p.bump(S!['{']);

    match p.peek() {
        Some(TokenKind::Name) => {
            object_field(p);
            if let Some(T!['}']) = p.peek() {
                p.bump(S!['}']);
            } else {
                p.err("expected }");
            }
        }
        Some(T!['}']) => {
            p.bump(S!['}']);
        }
        _ => p.err("expected Object Value"),
    }
}

/// See: https://spec.graphql.org/October2021/#ObjectField
///
/// *ObjectField*:
///     Name **:** Value
pub(crate) fn object_field(p: &mut Parser) {
    if let Some(TokenKind::Name) = p.peek() {
        let guard = p.start_node(SyntaxKind::OBJECT_FIELD);
        name::name(p);

        if let Some(T![:]) = p.peek() {
            p.bump(S![:]);
            value(p, true);
            if p.peek().is_some() {
                guard.finish_node();
                object_field(p)
            }
        }
    }
}

/// See: https://spec.graphql.org/October2021/#DefaultValue
///
/// *DefaultValue*:
///     **=** Value
pub(crate) fn default_value(p: &mut Parser) {
    let _g = p.start_node(SyntaxKind::DEFAULT_VALUE);
    p.bump(S![=]);
    value(p, false);
}

#[cfg(test)]
mod test {
    use crate::{ast, ast::AstNode, Parser};

    #[test]
    fn it_returns_string_for_string_value_into() {
        let schema = r#"
enum Test @dir__one(string: "string value", int_value: -10, float_value: -1.123e+4, bool: false) {
  INVENTORY
} "#;
        let parser = Parser::new(schema);
        let ast = parser.parse();

        assert!(ast.errors.is_empty());

        let document = ast.document();
        for definition in document.definitions() {
            if let ast::Definition::EnumTypeDefinition(enum_) = definition {
                for directive in enum_.directives().unwrap().directives() {
                    for argument in directive.arguments().unwrap().arguments() {
                        if let ast::Value::StringValue(val) =
                            argument.value().expect("Cannot get argument value.")
                        {
                            let source = val.source_string();
                            assert_eq!(source, r#""string value", "#); // SyntaxNodes include trailing
                                                                       // tokens like commas

                            let contents: String = val.into();
                            assert_eq!(contents, "string value");
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn it_returns_i64_for_int_values() {
        let schema = r#"
enum Test @dir__one(int_value: -10) {
  INVENTORY
} "#;
        let parser = Parser::new(schema);
        let ast = parser.parse();

        assert!(ast.errors.is_empty());

        let document = ast.document();
        for definition in document.definitions() {
            if let ast::Definition::EnumTypeDefinition(enum_) = definition {
                for directive in enum_.directives().unwrap().directives() {
                    for argument in directive.arguments().unwrap().arguments() {
                        if let ast::Value::IntValue(val) =
                            argument.value().expect("Cannot get argument value.")
                        {
                            let i: i32 = val.into();
                            assert_eq!(i, -10);
                        }
                    }
                }
            }
        }
    }

    #[test]
    // Allow only for this test, as this tests doesn't actually aim to compare
    // floats, but is here to ensure we are able to extract an f64 value
    #[allow(clippy::float_cmp)]
    fn it_returns_f64_for_float_values() {
        let schema = r#"
enum Test @dir__one(float_value: -1.123E4) {
  INVENTORY
} "#;
        let parser = Parser::new(schema);
        let ast = parser.parse();

        assert!(ast.errors.is_empty());

        let document = ast.document();
        for definition in document.definitions() {
            if let ast::Definition::EnumTypeDefinition(enum_) = definition {
                for directive in enum_.directives().unwrap().directives() {
                    for argument in directive.arguments().unwrap().arguments() {
                        if let ast::Value::FloatValue(val) =
                            argument.value().expect("Cannot get argument value.")
                        {
                            let f: f64 = val.into();
                            assert_eq!(f, -1.123E4);
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn it_returns_bool_for_boolean_values() {
        let schema = r#"
enum Test @dir__one(bool_value: false) {
  INVENTORY
} "#;
        let parser = Parser::new(schema);
        let ast = parser.parse();

        assert!(ast.errors.is_empty());

        let document = ast.document();
        for definition in document.definitions() {
            if let ast::Definition::EnumTypeDefinition(enum_) = definition {
                for directive in enum_.directives().unwrap().directives() {
                    for argument in directive.arguments().unwrap().arguments() {
                        if let ast::Value::BooleanValue(val) =
                            argument.value().expect("Cannot get argument value.")
                        {
                            let b: bool = val.into();
                            assert!(!b);
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn it_parses_variable_names() {
        let input = "
query GraphQuery($graph_id: ID!, $variant: String) {
  service(id: $graph_id) {
    schema(tag: $variant) {
      document
    }
  }
}
        ";
        let parser = Parser::new(input);
        let ast = parser.parse();
        assert_eq!(0, ast.errors().len());

        let doc = ast.document();

        for def in doc.definitions() {
            if let ast::Definition::OperationDefinition(op_def) = def {
                assert_eq!(op_def.name().unwrap().text(), "GraphQuery");

                let variable_defs = op_def.variable_definitions();
                let variables: Vec<String> = variable_defs
                    .iter()
                    .flat_map(|v| v.variable_definitions())
                    .filter_map(|v| Some(v.variable()?.text().to_string()))
                    .collect();
                assert_eq!(
                    variables.as_slice(),
                    ["graph_id".to_string(), "variant".to_string()]
                );
            }
        }
    }

    #[test]
    fn it_parse_mutation_with_escaped_char() {
        let input = r#"mutation {
            createStore(draft: {
              name: [{ locale: "en", value: "\"my store\"" }]
            }) {
              name(locale: "en")
            }
          }"#;
        let parser = Parser::new(input);
        let ast = parser.parse();

        assert!(ast.errors.is_empty());
    }

    #[test]
    fn it_parse_mutation_without_escaped_char() {
        let input = r#"mutation {
            createStore(draft: {
              name: [{ locale: "en", value: "my store" }]
            }) {
              name(locale: "en")
            }
          }"#;
        let parser = Parser::new(input);
        let ast = parser.parse();

        assert!(ast.errors.is_empty());
    }

    #[test]
    fn it_parse_mutation_without_escaped_char_with_error() {
        let input = r#"mutation {
            createStore(draft: {
              name: [{ locale: "en", value: "\"my store" }]
            }) {
              name(locale: "en")
            }
          }"#;
        let parser = Parser::new(input);
        let ast = parser.parse();

        assert!(ast.errors.is_empty());
    }

    #[test]
    fn it_parse_mutation_with_escaped_chars_and_without() {
        let input = r#"mutation {
            createStore(draft: {
              name: [{ locale: "en", value: "my \a store" }]
            }) {
              name(locale: "en")
            }
          }"#;
        let parser = Parser::new(input);
        let ast = parser.parse();

        assert!(!ast.errors.is_empty());
    }

    #[test]
    fn it_returns_error_for_unfinished_string_value_in_list() {
        let schema = r#"extend schema
  @link(url: "https://specs.apollo.dev/federation/v2.0",
        import: ["@key", "@external])
        
type Vehicle @key(fields: "id") {
  id: ID!,
  type: String,
  modelCode: String,
  brandName: String,
  launchDate: String
}
"#;

        let parser = Parser::new(schema);
        let ast = parser.parse();
        assert!(!ast.errors.is_empty());
    }
}
