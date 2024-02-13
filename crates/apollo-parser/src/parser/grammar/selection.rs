use std::ops::ControlFlow;

use crate::{
    parser::grammar::{field, fragment},
    Parser, SyntaxKind, TokenKind, S, T,
};

/// See: https://spec.graphql.org/October2021/#SelectionSet
///
/// *SelectionSet*:
///     **{** Selection* **}**
pub(crate) fn selection_set(p: &mut Parser) {
    if let Some(T!['{']) = p.peek() {
        let _g = p.start_node(SyntaxKind::SELECTION_SET);
        p.bump(S!['{']);

        // We need to enforce recursion limits to prevent
        // excessive resource consumption or (more seriously)
        // stack overflows.
        if p.recursion_limit.check_and_increment() {
            p.limit_err("parser recursion limit reached");
            return;
        }
        selection(p);
        p.recursion_limit.decrement();

        p.expect(T!['}'], S!['}']);
    }
}

pub(crate) fn field_set(p: &mut Parser) {
    if let Some(T!['{']) = p.peek() {
        selection_set(p)
    } else {
        let _g = p.start_node(SyntaxKind::SELECTION_SET);
        // We need to enforce recursion limits to prevent
        // excessive resource consumption or (more seriously)
        // stack overflows.
        if p.recursion_limit.check_and_increment() {
            p.limit_err("parser recursion limit reached");
            return;
        }
        selection(p);
        p.recursion_limit.decrement();
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

    p.peek_while(|p, kind| match kind {
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
                        p.bump(S![...]);
                    }
                    has_selection = true;
                    ControlFlow::Continue(())
                }
                None => {
                    p.err_and_pop("expected an Inline Fragment or a Fragment Spread");
                    ControlFlow::Break(())
                }
            }
        }
        T!['{'] => ControlFlow::Break(()),
        TokenKind::Name => {
            field::field(p);
            has_selection = true;
            ControlFlow::Continue(())
        }
        _ => ControlFlow::Break(()),
    });

    if !has_selection {
        p.err("expected at least one Selection in Selection Set");
    }
}

#[cfg(test)]
mod test {
    use crate::{cst, Parser, TokenText};

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
        let cst = parser.parse();
        assert_eq!(0, cst.errors().len());

        let doc = cst.document();

        for def in doc.definitions() {
            if let cst::Definition::OperationDefinition(op_def) = def {
                if let Some(selection_set) = op_def.selection_set() {
                    for selection in selection_set.selections() {
                        match selection {
                            cst::Selection::Field(field) => {
                                assert_eq!(
                                    "animal".to_string(),
                                    field.name().unwrap().text().to_string()
                                );
                            }
                            cst::Selection::FragmentSpread(f_spread) => {
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
                            cst::Selection::InlineFragment(inline_fragment) => {
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
        let cst = parser.parse();
        assert_eq!(0, cst.errors().len());

        let doc = cst.document();

        for def in doc.definitions() {
            if let cst::Definition::OperationDefinition(op_def) = def {
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
            selection_set: cst::SelectionSet,
        ) -> &Vec<TokenText> {
            for selection in selection_set.selections() {
                match selection {
                    cst::Selection::Field(field) => {
                        let arguments = field.arguments();
                        let mut vars: Vec<TokenText> = arguments
                            .iter()
                            .flat_map(|a| a.arguments())
                            .filter_map(|v| {
                                if let cst::Value::Variable(var) = v.value()? {
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
        let cst = parser.parse();

        assert_eq!(cst.errors().len(), 0);

        let doc = cst.document();
        for def in doc.definitions() {
            if let cst::Definition::OperationDefinition(op_def) = def {
                let selection_set = op_def.selection_set().unwrap();
                for selection in selection_set.selections() {
                    if let cst::Selection::Field(field) = selection {
                        assert_eq!("item1", field.name().unwrap().text().as_ref());
                        let selection_set = field.selection_set().unwrap();
                        for selection in selection_set.selections() {
                            match selection {
                                cst::Selection::Field(field) => {
                                    assert_eq!("id", field.name().unwrap().text().as_ref());
                                }
                                cst::Selection::FragmentSpread(fragment_spread) => {
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
                                cst::Selection::InlineFragment(inline_fragment) => {
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

        let cst = parser.parse();

        assert_eq!(cst.errors().len(), 1);
        assert_eq!(cst.document().definitions().count(), 1);
    }

    #[test]
    fn it_errors_when_selection_set_recursion_limit_exceeded() {
        let schema = r#"
        query {
          Q1 {
            Q2 {
              url
            }
          }
        }
        "#;
        let parser = Parser::new(schema).recursion_limit(1);

        let cst = parser.parse();

        assert_eq!(cst.recursion_limit().high, 2);
        assert_eq!(cst.errors().len(), 1);
        assert_eq!(cst.document().definitions().count(), 1);
    }

    #[test]
    fn it_passes_when_selection_set_recursion_limit_is_not_exceeded() {
        let schema = r#"
        query {
          Q1 {
            Q2 {
              url
            }
          }
        }
        "#;
        let parser = Parser::new(schema).recursion_limit(7);

        let cst = parser.parse();

        assert_eq!(cst.recursion_limit().high, 3);
        assert_eq!(cst.errors().len(), 0);
        assert_eq!(cst.document().definitions().count(), 1);
    }

    #[test]
    fn it_errors_when_selection_set_recursion_limit_is_exceeded_with_inline_fragment() {
        let schema = r#"
        query {
          Q1 {
            Q2 {
              ... on Page {
                price
                name
              }
            }
          }
        }
        "#;
        let parser = Parser::new(schema).recursion_limit(2);

        let cst = parser.parse();

        assert_eq!(cst.recursion_limit().high, 3);
        assert_eq!(cst.errors().len(), 1);
        assert_eq!(cst.document().definitions().count(), 1);
    }

    #[test]
    fn it_errors_when_selection_set_recursion_limit_is_exceeded_fragment_spread() {
        let schema = r#"
        query {
          Q1 {
            Q2 {
              product {
                ...Page
              }
            }
          }
        }
        "#;
        let parser = Parser::new(schema).recursion_limit(2);

        let cst = parser.parse();

        assert_eq!(cst.recursion_limit().high, 3);
        assert_eq!(cst.errors().len(), 1);
        assert_eq!(dbg!(cst.document()).definitions().count(), 1);
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
        let parser = Parser::new(schema).recursion_limit(1);

        let cst = parser.parse();

        assert_eq!(cst.recursion_limit().high, 2);
        assert_eq!(cst.errors().len(), 1);
        assert_eq!(cst.document().definitions().count(), 2);
    }

    #[test]
    fn it_passes_when_selection_set_recursion_limit_is_at_default() {
        let schema = r#"
        query {
          Q1 {
            Q2 {
              Q3 {
                Q4 {
                  Q5 { url }
                }
              }
            }
          },
        }
        "#;
        let parser = Parser::new(schema);

        let cst = parser.parse();

        assert_eq!(cst.recursion_limit().high, 6);
        assert_eq!(cst.errors().len(), 0);
        assert_eq!(cst.document().definitions().count(), 1);
    }
}
