use std::collections::HashMap;

use crate::{
    diagnostics::{CapitalizedValue, UniqueDefinition},
    hir::EnumValueDefinition,
    ApolloDiagnostic, Validation,
};

pub fn check(db: &dyn Validation) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = Vec::new();

    // An Enum type must define one or more unique enum values.
    //
    // Return a Unique Value error in case of a duplicate value.
    for enum_def in db.enums().iter() {
        let mut seen: HashMap<&str, &EnumValueDefinition> = HashMap::new();
        for enum_value in enum_def.enum_values_definition().iter() {
            let value = enum_value.enum_value();
            if let Some(prev_def) = seen.get(&value) {
                let prev_offset: usize = prev_def.ast_node(db.upcast()).text_range().start().into();
                let prev_node_len: usize = prev_def.ast_node(db.upcast()).text_range().len().into();

                let current_offset: usize =
                    enum_value.ast_node(db.upcast()).text_range().start().into();
                let current_node_len: usize =
                    enum_value.ast_node(db.upcast()).text_range().len().into();
                diagnostics.push(ApolloDiagnostic::UniqueDefinition(UniqueDefinition {
                    ty: "enum".into(),
                    name: value.into(),
                    src: db.input(),
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
            let offset: usize = enum_value.ast_node(db.upcast()).text_range().start().into();
            let len: usize = enum_value.ast_node(db.upcast()).text_range().len().into();

            if value.to_uppercase() != value {
                diagnostics.push(ApolloDiagnostic::CapitalizedValue(CapitalizedValue {
                    ty: value.into(),
                    src: db.input(),
                    value: (offset, len).into(),
                }));
            }
        }
    }

    diagnostics
}
