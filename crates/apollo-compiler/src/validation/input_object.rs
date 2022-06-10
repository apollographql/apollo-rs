use std::collections::HashSet;

use crate::{diagnostics::ErrorDiagnostic, ApolloDiagnostic, SourceDatabase};

pub fn check(db: &dyn SourceDatabase) -> Vec<ApolloDiagnostic> {
    let mut errors = Vec::new();

    // Input Object Definitions must have unique names.
    //
    // Return a Unique Definition error in case of a duplicate name.
    let mut seen = HashSet::new();
    for input_objects in db.input_objects().iter() {
        let name = input_objects.name();
        if seen.contains(name) {
            errors.push(ApolloDiagnostic::Error(ErrorDiagnostic::UniqueDefinition {
                message: "Input Object Definitions must have unique names".into(),
                definition: name.to_string(),
            }));
        } else {
            seen.insert(name);
        }
    }

    // Fields in an Input Object Definition must be unique
    //
    // Returns Unique Value error.
    for input_objects in db.input_objects().iter() {
        let mut seen = HashSet::new();

        let input_fields = input_objects.input_fields_definition();

        for field in input_fields {
            let field_name = field.name();
            if seen.contains(&field_name) {
                errors.push(ApolloDiagnostic::Error(ErrorDiagnostic::UniqueValue {
                    message: "Input Fields must be unique".into(),
                    value: field_name.into(),
                }));
            } else {
                seen.insert(field_name);
            }
        }
    }
    errors
}
