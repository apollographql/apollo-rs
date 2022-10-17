use crate::{
    parser::grammar::{field, fragment},
    Parser, SyntaxKind, TokenKind, S, T,
};

/// In order to control recursion, we need to have a way to
/// specify the "top" of a selection_set tree. This is the
/// function which must be used to do this.
pub(crate) fn top_selection_set(p: &mut Parser) {
    p.recursion_limit.reset();
    selection_set(p)
}

/// See: https://spec.graphql.org/October2021/#SelectionSet
///
/// *SelectionSet*:
///     **{** Selection* **}**
pub(crate) fn selection_set(p: &mut Parser) {
    p.recursion_limit.consume();

    if let Some(T!['{']) = p.peek() {
        let _g = p.start_node(SyntaxKind::SELECTION_SET);
        p.bump(S!['{']);
        // We need to enforce recursion limits to prevent
        // excessive resource consumption or (more seriously)
        // stack overflows.
        if p.recursion_limit.limited() {
            p.limit_err(format!("parser limit({}) reached", p.recursion_limit.limit));
            return;
        }
        selection(p);
        p.expect(T!['}'], S!['}']);
    }
}

/// See: https://spec.graphql.org/October2021/#Selection
///
/// *Selection*:
///     Field
///     FragmentSpread
///     InlineFragment
pub(crate) fn selection(p: &mut Parser) {
    let mut has_selection = false;

    while let Some(node) = p.peek() {
        if p.recursion_limit.limited() {
            break;
        }
        match node {
            T![...] => {
                let next_token = p.peek_token_n(2);
                match next_token {
                    Some(next_token) => {
                        if next_token.kind() == TokenKind::Name && next_token.data() != "on" {
                            fragment::fragment_spread(p);
                        } else if matches!(
                            next_token.kind(),
                            TokenKind::At | TokenKind::Name | TokenKind::LCurly
                        ) {
                            fragment::inline_fragment(p);
                        } else {
                            p.err("expected an Inline Fragment or a Fragment Spread");
                        }
                        has_selection = true;
                    }
                    None => p.err("expected an Inline Fragment or a Fragment Spread"),
                }
            }
            T!['{'] => {
                break;
            }
            TokenKind::Name => {
                field::field(p);
                has_selection = true;
            }
            _ => {
                if !has_selection {
                    p.err("expected at least one Selection in Selection Set");
                }
                break;
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{ast, Parser, TokenText};

    #[test]
    fn fragment_spread_in_selection() {
        let input = "
{
    animal
    ...snackSelection
    ... on Pet {
      playmates {
        count
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
                if let Some(selection_set) = op_def.selection_set() {
                    for selection in selection_set.selections() {
                        match selection {
                            ast::Selection::Field(field) => {
                                assert_eq!(
                                    "animal".to_string(),
                                    field.name().unwrap().text().to_string()
                                );
                            }
                            ast::Selection::FragmentSpread(f_spread) => {
                                assert_eq!(
                                    "snackSelection".to_string(),
                                    f_spread
                                        .fragment_name()
                                        .unwrap()
                                        .name()
                                        .unwrap()
                                        .text()
                                        .to_string()
                                )
                            }
                            ast::Selection::InlineFragment(inline_fragment) => {
                                assert_eq!(
                                    "Pet".to_string(),
                                    inline_fragment
                                        .type_condition()
                                        .unwrap()
                                        .named_type()
                                        .unwrap()
                                        .name()
                                        .unwrap()
                                        .text()
                                        .to_string()
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn do_query_variables_match() {
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
                let variables: Vec<TokenText> = variable_defs
                    .iter()
                    .flat_map(|v| v.variable_definitions())
                    .filter_map(|v| Some(v.variable()?.name()?.text()))
                    .collect();

                if let Some(selection_set) = op_def.selection_set() {
                    let mut vec = Vec::default();
                    let used_vars = get_variables_from_selection(&mut vec, selection_set);
                    assert!(do_variables_match(&variables, used_vars));
                }
            }
        }

        fn get_variables_from_selection(
            used_vars: &mut Vec<TokenText>,
            selection_set: ast::SelectionSet,
        ) -> &Vec<TokenText> {
            for selection in selection_set.selections() {
                match selection {
                    ast::Selection::Field(field) => {
                        let arguments = field.arguments();
                        let mut vars: Vec<TokenText> = arguments
                            .iter()
                            .flat_map(|a| a.arguments())
                            .filter_map(|v| {
                                if let ast::Value::Variable(var) = v.value()? {
                                    return Some(var.name()?.text());
                                }
                                None
                            })
                            .collect();
                        used_vars.append(&mut vars);
                        if let Some(selection_set) = field.selection_set() {
                            get_variables_from_selection(used_vars, selection_set);
                        }
                    }
                    _ => unimplemented!(),
                }
            }
            used_vars
        }

        fn do_variables_match(a: &[TokenText], b: &[TokenText]) -> bool {
            let matching = a.iter().zip(b.iter()).filter(|&(a, b)| a == b).count();
            matching == a.len() && matching == b.len()
        }
    }

    #[test]
    fn it_gets_nested_selection_set_fields() {
        let query = r#"
query SomeQuery(
  $param1: String!
  $param2: String!
) {
  item1(
    param1: $param1
    param2: $param2
  ) {
    id
    ...Fragment1
    ... on Fragment2 {
      field3 {
        field4
      }
    }
  }
}"#;
        let parser = Parser::new(query);
        let ast = parser.parse();

        assert_eq!(ast.errors().len(), 0);

        let doc = ast.document();
        for def in doc.definitions() {
            if let ast::Definition::OperationDefinition(op_def) = def {
                let selection_set = op_def.selection_set().unwrap();
                for selection in selection_set.selections() {
                    if let ast::Selection::Field(field) = selection {
                        assert_eq!("item1", field.name().unwrap().text().as_ref());
                        let selection_set = field.selection_set().unwrap();
                        for selection in selection_set.selections() {
                            match selection {
                                ast::Selection::Field(field) => {
                                    assert_eq!("id", field.name().unwrap().text().as_ref());
                                }
                                ast::Selection::FragmentSpread(fragment_spread) => {
                                    assert_eq!(
                                        "Fragment1",
                                        fragment_spread
                                            .fragment_name()
                                            .unwrap()
                                            .name()
                                            .unwrap()
                                            .text()
                                            .as_ref()
                                    );
                                }
                                ast::Selection::InlineFragment(inline_fragment) => {
                                    assert_eq!(
                                        "Fragment2",
                                        inline_fragment
                                            .type_condition()
                                            .unwrap()
                                            .named_type()
                                            .unwrap()
                                            .name()
                                            .unwrap()
                                            .text()
                                            .as_ref()
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn it_errors_when_selection_set_is_empty() {
        let schema = r#"
        query($foo: Int) {}
        "#;
        let parser = Parser::new(schema);

        let ast = parser.parse();

        assert_eq!(ast.errors().len(), 1);
        assert_eq!(ast.document().definitions().into_iter().count(), 1);
    }

    #[test]
    fn it_errors_when_selection_set_recursion_limit_exceeded() {
        let schema = r#"
        query {
          Q1 {
            url
          }
        }
        "#;
        let parser = Parser::with_recursion_limit(schema, 1);

        let ast = parser.parse();

        assert_eq!(ast.recursion_limit().high, 2);
        assert_eq!(ast.errors().len(), 1);
        assert_eq!(ast.document().definitions().into_iter().count(), 2);
    }

    #[test]
    fn it_passes_when_selection_set_recursion_limit_is_not_exceeded() {
        let schema = r#"
        query {
          Q1 {
            url
          }
        }
        "#;
        let parser = Parser::with_recursion_limit(schema, 7);

        let ast = parser.parse();

        assert_eq!(ast.recursion_limit().high, 4);
        assert_eq!(ast.errors().len(), 0);
        assert_eq!(ast.document().definitions().into_iter().count(), 1);
    }

    #[test]
    fn it_errors_when_selection_set_recursion_limit_is_exceeded_with_inline_fragment() {
        let schema = r#"
        query {
          ... on Page {
            price
            name
          }
        }
        "#;
        let parser = Parser::with_recursion_limit(schema, 2);

        let ast = parser.parse();

        assert_eq!(ast.recursion_limit().high, 3);
        assert_eq!(ast.errors().len(), 1);
        assert_eq!(ast.document().definitions().into_iter().count(), 1);
    }

    #[test]
    fn it_errors_when_selection_set_recursion_limit_is_exceeded_fragment_spread() {
        let schema = r#"
        query {
          product {
            ...Page
          }
        }
        "#;
        let parser = Parser::with_recursion_limit(schema, 2);

        let ast = parser.parse();

        assert_eq!(ast.recursion_limit().high, 3);
        assert_eq!(ast.errors().len(), 1);
        assert_eq!(ast.document().definitions().into_iter().count(), 1);
    }

    #[test]
    fn it_errors_when_selection_set_recursion_limit_is_exceeded_in_fragment_definition() {
        let schema = r#"
        query ExampleQuery {
          topProducts {
            name
          }
          ... multipleSubscriptions
        }

        fragment multipleSubscriptions on Subscription {
          newMessage {
            body
            sender
          }
        }
        "#;
        let parser = Parser::with_recursion_limit(schema, 1);

        let ast = parser.parse();

        assert_eq!(ast.recursion_limit().high, 2);
        assert_eq!(ast.errors().len(), 1);
        assert_eq!(ast.document().definitions().into_iter().count(), 4);
    }

    #[test]
    fn it_passes_when_selection_set_recursion_limit_is_at_default() {
        let schema = r#"
        query {
          Q1 {
            url
          },
        }
        "#;
        let parser = Parser::new(schema);

        let ast = parser.parse();

        assert_eq!(ast.recursion_limit().high, 4);
        assert_eq!(ast.errors().len(), 0);
        assert_eq!(ast.document().definitions().into_iter().count(), 1);
    }
}
