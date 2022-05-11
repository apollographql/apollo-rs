use std::collections::HashSet;

use crate::{diagnostics::ErrorDiagnostic, ApolloDiagnostic, SourceDatabase};

pub fn check(db: &dyn SourceDatabase) -> Vec<ApolloDiagnostic> {
    let mut errors = Vec::new();

    // A GraphQL schema must have a Query root operation.
    if db.schema().query(db).is_none() {
        let error = ApolloDiagnostic::Error(ErrorDiagnostic::QueryRootOperationType(
            "Missing query root operation type in schema definition".to_string(),
        ));
        errors.push(error);
    }

    // All root operations in a schema definition must be unique.
    //
    // Return a Unique Operation Definition error in case of a duplicate name.
    let mut seen = HashSet::new();
    for op_type in db.schema().root_operation_type_definition().iter() {
        let name = op_type.named_type().name();
        if seen.contains(&name) {
            errors.push(ApolloDiagnostic::Error(
                ErrorDiagnostic::UniqueRootOperationType {
                    message: "Root Operation Types must be unique".into(),
                    operation_type: op_type.operation_type().to_string(),
                    named_type: name,
                },
            ));
        } else {
            seen.insert(name);
        }
    }

    errors
}
