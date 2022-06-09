use std::collections::HashSet;

use crate::{diagnostics::ErrorDiagnostic, ApolloDiagnostic, SourceDatabase};

pub fn check(db: &dyn SourceDatabase) -> Vec<ApolloDiagnostic> {
    let mut errors = Vec::new();

    // Directive definitions must have unique names.
    //
    // Return a Unique Definition error in case of a duplicate name.
    let mut seen = HashSet::new();
    for directive_definitions in db.directive_definitions().iter() {
        let name = directive_definitions.name();
        if seen.contains(name) {
            errors.push(ApolloDiagnostic::Error(ErrorDiagnostic::UniqueDefinition {
                message: "Directive Definitions must have unique names".into(),
                definition: name.to_string(),
            }));
        } else {
            seen.insert(name);
        }
    }

    // A directive definition must not contain the use of a directive which
    // references itself directly.
    //
    // Returns Recursive Definition error.
    for directive_def in db.directive_definitions().iter() {
        let name = directive_def.name();
        for input_values in directive_def.arguments().input_values() {
            for directive in input_values.directives().iter() {
                let directive_name = directive.name();
                if name == directive_name {
                    errors.push(ApolloDiagnostic::Error(
                        ErrorDiagnostic::RecursiveDefinition {
                            message: "Directive definition cannot reference itself".into(),
                            definition: directive_name.to_string(),
                        },
                    ));
                }
            }
        }
    }

    errors
}
