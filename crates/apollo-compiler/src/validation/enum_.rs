use std::collections::HashMap;

use crate::{
    diagnostics::{CapitalizedValue, UniqueDefinition},
    hir::EnumValueDefinition,
    ApolloDiagnostic, ValidationDatabase,
};

pub fn check(db: &dyn ValidationDatabase) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = Vec::new();

    // An Enum type must define one or more unique enum values.
    //
    // Return a Unique Value error in case of a duplicate value.
    for enum_def in db.enums().iter() {
        let mut seen: HashMap<&str, &EnumValueDefinition> = HashMap::new();
        for enum_value in enum_def.enum_values_definition().iter() {
            let value = enum_value.enum_value();
            if let Some(prev_def) = seen.get(&value) {
                let prev_offset = prev_def.loc().offset();
                let prev_node_len = prev_def.loc().node_len();

                let current_offset = enum_value.loc().offset();
                let current_node_len = enum_value.loc().node_len();
                diagnostics.push(ApolloDiagnostic::UniqueDefinition(UniqueDefinition {
                    ty: "enum".into(),
                    name: value.into(),
                    src: db.source_code(prev_def.loc().file_id()),
                    original_definition: (prev_offset, prev_node_len).into(),
                    redefined_definition: (current_offset, current_node_len).into(),
                    help: Some(format!("{value} must only be defined once in this enum.")),
                }));
            } else {
                seen.insert(value, enum_value);
            }
        }
    }

    // (convention) Values in an Enum Definition should be capitalized.
    //
    // Return a Capitalized Value warning if enum value is not capitalized.
    for enum_def in db.enums().iter() {
        for enum_value in enum_def.enum_values_definition().iter() {
            let value = enum_value.enum_value();
            let offset = enum_value.loc().offset();
            let len = enum_value.loc().node_len();

            if value.to_uppercase() != value {
                diagnostics.push(ApolloDiagnostic::CapitalizedValue(CapitalizedValue {
                    ty: value.into(),
                    src: db.source_code(enum_value.loc().file_id()),
                    value: (offset, len).into(),
                }));
            }
        }
    }

    diagnostics
}
