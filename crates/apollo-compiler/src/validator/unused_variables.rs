use crate::{diagnostics::ApolloDiagnostic, SourceDatabase};

pub fn check(db: &dyn SourceDatabase) -> Vec<ApolloDiagnostic> {
    db.operations()
        .iter()
        .flat_map(|op| {
            let defined_vars = db.operation_definition_defined_variables(op.id()).unwrap();
            let used_vars = db.operation_definition_in_use_variables(op.id()).unwrap();
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
        })
        .collect()
}
