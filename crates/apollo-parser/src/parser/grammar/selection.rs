use crate::{
    parser::grammar::{field, fragment},
    Parser, SyntaxKind, TokenKind, S, T,
};

/// See: https://spec.graphql.org/draft/#SelectionSet
///
/// *SelectionSet*:
///     **{** Selection<sub>list</sub> **}**
pub(crate) fn selection_set(p: &mut Parser) {
    if let Some(T!['{']) = p.peek() {
        let _g = p.start_node(SyntaxKind::SELECTION_SET);
        p.bump(S!['{']);
        selection(p);
        p.expect(T!['}'], S!['}']);
    }
}

/// See: https://spec.graphql.org/draft/#Selection
///
/// *Selection*:
///     Field
///     FragmentSpread
///     InlineFragment
pub(crate) fn selection(p: &mut Parser) {
    while let Some(node) = p.peek() {
        match node {
            T![...] => {
                if let Some(node) = p.peek_data_n(2) {
                    match node.as_str() {
                        "on" | "{" => fragment::inline_fragment(p),
                        _ => fragment::fragment_spread(p),
                    }
                } else {
                    p.err("expected an Inline Fragment or a Fragment Spread");
                }
            }
            T!['{'] => {
                break;
            }
            TokenKind::Name => {
                field::field(p);
            }
            _ => break,
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{
        ast::{Definition, Selection, SelectionSet, Value},
        Parser, TokenText,
    };

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
        assert!(&ast.errors().is_empty());

        let doc = ast.document();

        for def in doc.definitions() {
            if let Definition::OperationDefinition(op_def) = def {
                if let Some(selection_set) = op_def.selection_set() {
                    for selection in selection_set.selections() {
                        match selection {
                            Selection::Field(field) => {
                                assert_eq!(
                                    "animal".to_string(),
                                    field.name().unwrap().text().to_string()
                                );
                            }
                            Selection::FragmentSpread(f_spread) => {
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
                            Selection::InlineFragment(inline_fragment) => {
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
        assert!(&ast.errors().is_empty());

        let doc = ast.document();

        for def in doc.definitions() {
            if let Definition::OperationDefinition(op_def) = def {
                assert_eq!(op_def.name().unwrap().text(), "GraphQuery");

                let variable_defs = op_def.variable_definitions();
                let variables: Vec<TokenText> = variable_defs
                    .iter()
                    .map(|v| v.variable_definitions())
                    .flatten()
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
            selection_set: SelectionSet,
        ) -> &Vec<TokenText> {
            for selection in selection_set.selections() {
                match selection {
                    Selection::Field(field) => {
                        let arguments = field.arguments();
                        let mut vars: Vec<TokenText> = arguments
                            .iter()
                            .map(|a| a.arguments())
                            .flatten()
                            .filter_map(|v| {
                                if let Value::Variable(var) = v.value()? {
                                    dbg!(&var.name()?.text());
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
}
