use crate::{diagnostics::ApolloDiagnostic, SourceDatabase};

pub fn check(db: &dyn SourceDatabase) -> Vec<ApolloDiagnostic> {
    db.operations()
        .iter()
        .flat_map(|op| {
            dbg!(&op);
            if let Some(name) = op.name() {
                let defined_vars = db
                    .operation_definition_defined_variables(name.clone())
                    .unwrap();
                let used_vars = db.operation_definition_in_use_variables(name).unwrap();
                let undefined_vars = used_vars.difference(&defined_vars);
                let mut diagnostics: Vec<ApolloDiagnostic> = undefined_vars
                    .map(|undefined_var| ApolloDiagnostic::UndefinedVariablesError {
                        message: "Variable undefined".into(),
                        variable: undefined_var.into(),
                    })
                    .collect();

                let unused_vars = defined_vars.difference(&used_vars);
                let warnings: Vec<ApolloDiagnostic> = unused_vars
                    .map(|unused_var| ApolloDiagnostic::UnusedVariablesWarning {
                        message: "unused variable".into(),
                        variable: unused_var.into(),
                    })
                    .collect();

                diagnostics.extend(warnings);
                diagnostics
            } else {
                Vec::new()
            }
        })
        .collect()
    // for def in doc.definitions() {
    //     if let ast::Definition::OperationDefinition(op_def) = def {
    //         let variable_defs = op_def.variable_definitions();
    //         // We grab all the variables defined in the mutation
    //         let defined_variables: Vec<String> = variable_defs
    //             .iter()
    //             .map(|v| v.variable_definitions())
    //             .flatten()
    //             .filter_map(|v| Some(v.variable()?.text().to_string()))
    //             .collect();

    //         if let Some(selection_set) = op_def.selection_set() {
    //             let mut vec = Vec::default();
    //             // Get the variables defined in the mutation's selection set.
    //             let used_vars = get_variables_from_selection(&mut vec, selection_set);
    //             // Compare the two sets of variables.
    //             assert!(do_variables_match(&defined_variables, used_vars));
    //         }
    //     }
    // }
}

// fn get_variables_from_selection(
//     used_vars: &mut Vec<String>,
//     selection_set: ast::SelectionSet,
// ) -> &Vec<String> {
//     for selection in selection_set.selections() {
//         match selection {
//             ast::Selection::Field(field) => {
//                 let arguments = field.arguments();
//                 let mut vars: Vec<String> = arguments
//                     .iter()
//                     .map(|a| a.arguments())
//                     .flatten()
//                     .filter_map(|v| {
//                         if let ast::Value::Variable(var) = v.value()? {
//                             return Some(var.text().to_string());
//                         }
//                         None
//                     })
//                     .collect();
//                 used_vars.append(&mut vars);
//                 if let Some(selection_set) = field.selection_set() {
//                     get_variables_from_selection(used_vars, selection_set);
//                 }
//             }
//             _ => unimplemented!(),
//         }
//     }
//     used_vars
// }
//
// fn do_variables_match(a: &[String], b: &[String]) -> bool {
//     let matching = a.iter().zip(b.iter()).filter(|&(a, b)| a == b).count();
//     matching == a.len() && matching == b.len()
// }
