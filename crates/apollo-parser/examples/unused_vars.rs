/// Example usage of `apollo-parser` to check for unused vars in a given GraphQL
/// query.
use std::{fs, path::Path};

use apollo_parser::{ast, Parser};

fn are_variables_unused() {
    // Example mutation with variables.
    let file = Path::new("crates/apollo-parser/examples/graph_check_mutation.graphql");
    let src = fs::read_to_string(file).expect("Could not read schema file.");
    let parser = Parser::new(&src);
    let ast = parser.parse();

    assert_eq!(0, ast.errors().len());

    let doc = ast.document();

    for def in doc.definitions() {
        if let ast::Definition::OperationDefinition(op_def) = def {
            assert_eq!(op_def.name().unwrap().text(), "GraphCheckMutation");

            let variable_defs = op_def.variable_definitions();
            // We grab all the variables defined in the mutation
            let variables: Vec<String> = variable_defs
                .iter()
                .map(|v| v.variable_definitions())
                .flatten()
                .filter_map(|v| Some(v.variable()?.text().to_string()))
                .collect();

            if let Some(selection_set) = op_def.selection_set() {
                let mut vec = Vec::default();
                // Get the variables defined in the mutation's selection set.
                let used_vars = get_variables_from_selection(&mut vec, selection_set);
                // Compare the two sets of variables.
                assert!(do_variables_match(&variables, used_vars));
            }
        }
    }
}

fn get_variables_from_selection(
    used_vars: &mut Vec<String>,
    selection_set: ast::SelectionSet,
) -> &Vec<String> {
    for selection in selection_set.selections() {
        match selection {
            ast::Selection::Field(field) => {
                let arguments = field.arguments();
                let mut vars: Vec<String> = arguments
                    .iter()
                    .map(|a| a.arguments())
                    .flatten()
                    .filter_map(|v| {
                        if let ast::Value::Variable(var) = v.value()? {
                            return Some(var.text().to_string());
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

fn do_variables_match(a: &[String], b: &[String]) -> bool {
    let matching = a.iter().zip(b.iter()).filter(|&(a, b)| a == b).count();
    matching == a.len() && matching == b.len()
}

fn main() {
    are_variables_unused();
}
