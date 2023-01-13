use std::collections::HashMap;

use crate::{
    diagnostics::{ApolloDiagnostic, DiagnosticData, Label},
    hir::EnumValueDefinition,
    ValidationDatabase,
};

pub fn check(db: &dyn ValidationDatabase) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = Vec::new();

    // An Enum type must define one or more unique enum values.
    //
    // Return a Unique Value error in case of a duplicate value.
    for enum_def in db.enums().values() {
        let mut seen: HashMap<&str, &EnumValueDefinition> = HashMap::new();
        for enum_value in enum_def.enum_values_definition().iter() {
            let value = enum_value.enum_value();
            if let Some(prev_def) = seen.get(&value) {
                let original_definition = prev_def.loc();
                let redefined_definition = enum_value.loc();
                diagnostics.push(
                    ApolloDiagnostic::new(
                        db,
                        redefined_definition.into(),
                        DiagnosticData::UniqueDefinition {
                            ty: "enum",
                            name: value.into(),
                            original_definition: original_definition.into(),
                            redefined_definition: redefined_definition.into(),
                        },
                    )
                    .labels([
                        Label::new(
                            original_definition,
                            format!("previous definition of `{value}` here"),
                        ),
                        Label::new(redefined_definition, format!("`{value}` redefined here")),
                    ])
                    .help(format!("{value} must only be defined once in this enum.")),
                );
            } else {
                seen.insert(value, enum_value);
            }
        }
    }

    // (convention) Values in an Enum Definition should be capitalized.
    //
    // Return a Capitalized Value warning if enum value is not capitalized.
    for enum_def in db.enums().values() {
        for enum_value in enum_def.enum_values_definition().iter() {
            let value = enum_value.enum_value();
            if value.to_uppercase() != value {
                diagnostics.push(
                    ApolloDiagnostic::new(
                        db,
                        enum_value.loc().into(),
                        DiagnosticData::CapitalizedValue {
                            value: value.into(),
                        },
                    )
                    .label(Label::new(
                        enum_value.loc(),
                        format!("consider capitalizing {value}"),
                    )),
                );
            }
        }
    }

    diagnostics
}
