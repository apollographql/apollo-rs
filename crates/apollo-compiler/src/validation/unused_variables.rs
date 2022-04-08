use crate::{
    diagnostics::{ApolloDiagnostic, ErrorDiagnostic, WarningDiagnostic},
    SourceDatabase,
};

pub fn check(db: &dyn SourceDatabase) -> Vec<ApolloDiagnostic> {
    db.operations()
        .iter()
        .filter_map(|op| {
            let defined_vars = db.operation_definition_defined_variables(op.id())?;
            let used_vars = db.operation_definition_in_use_variables(op.id())?;
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
            let warnings: Vec<ApolloDiagnostic> = unused_vars
                .map(|unused_var| {
                    ApolloDiagnostic::Warning(WarningDiagnostic::UnusedVariable {
                        message: "unused variable".into(),
                        variable: unused_var.into(),
                    })
                })
                .collect();

            diagnostics.extend(warnings);
            Some(diagnostics)
        })
        .flatten()
        .collect()
}
