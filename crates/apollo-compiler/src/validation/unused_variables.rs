use crate::{
    diagnostics::{ApolloDiagnostic, ErrorDiagnostic, WarningDiagnostic},
    SourceDatabase,
};

// check in scope
// check in use
// compare the two
pub fn check(db: &dyn SourceDatabase) -> Vec<ApolloDiagnostic> {
    db.operations()
        .iter()
        .flat_map(|op| {
            let defined_vars = db
                .operation_definition_variables(op.id())
                .unwrap_or_default();
            let used_vars = db.selection_variables(op.id()).unwrap_or_default();
            let undefined_vars = used_vars.difference(&defined_vars);
            let mut diagnostics: Vec<ApolloDiagnostic> = undefined_vars
                .map(|undefined_var| {
                    ApolloDiagnostic::Error(ErrorDiagnostic::UndefinedVariable {
                        message: "Variable undefined".into(),
                        variable: undefined_var.into(),
                    })
                })
                .collect();

            let unused_vars = defined_vars.difference(&used_vars);
            let warnings = unused_vars.map(|unused_var| {
                ApolloDiagnostic::Warning(WarningDiagnostic::UnusedVariable {
                    message: "unused variable".into(),
                    variable: unused_var.into(),
                })
            });

            diagnostics.extend(warnings);
            diagnostics
        })
        .collect()
}
