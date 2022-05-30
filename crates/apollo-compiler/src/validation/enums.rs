use std::collections::HashSet;

use crate::{
    diagnostics::{ErrorDiagnostic, WarningDiagnostic},
    ApolloDiagnostic, SourceDatabase,
};

pub fn check(db: &dyn SourceDatabase) -> Vec<ApolloDiagnostic> {
    let mut errors = Vec::new();

    // An Enum type must define one or more unique enum values.
    //
    // Return a Unique Value error in case of a duplicate value.
    for enum_def in db.enums().iter() {
        let mut seen = HashSet::new();
        for enum_value in enum_def.enum_values_definition().iter() {
            let value = enum_value.enum_value();
            if seen.contains(&value) {
                errors.push(ApolloDiagnostic::Error(ErrorDiagnostic::UniqueValue {
                    message: "Enum Definition must have unique values".into(),
                    value: value.into(),
                }));
            } else {
                seen.insert(value);
            }
        }
    }

    // (convention) Values in an Enum Definition should be capitalized.
    //
    // Return a Capitalized Value warning if enum value is not capitalized.
    for enum_def in db.enums().iter() {
        for enum_value in enum_def.enum_values_definition().iter() {
            let value = enum_value.enum_value();
            if value.to_uppercase() != value {
                errors.push(ApolloDiagnostic::Warning(
                    WarningDiagnostic::CapitalizedValue {
                        message: "Enum values are recommended to be capitalized".into(),
                        value: value.into(),
                    },
                ));
            }
        }
    }

    errors
}
