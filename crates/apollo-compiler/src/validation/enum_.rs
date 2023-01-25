use std::collections::HashMap;

use crate::{
    diagnostics::{CapitalizedValue, UniqueDefinition},
    hir::{self, EnumValueDefinition},
    ApolloDiagnostic, ValidationDatabase,
};

pub fn validate_enum_definitions(db: &dyn ValidationDatabase) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = Vec::new();

    let defs = &db.type_system_definitions().enums;
    for def in defs.values() {
        diagnostics.extend(db.validate_enum_definition(def.as_ref().clone()));
    }

    diagnostics
}

pub fn validate_enum_definition(
    db: &dyn ValidationDatabase,
    enum_def: hir::EnumTypeDefinition,
) -> Vec<ApolloDiagnostic> {
    let mut diagnostics =
        db.validate_directives(enum_def.directives().to_vec(), hir::DirectiveLocation::Enum);

    let mut seen: HashMap<&str, &EnumValueDefinition> = HashMap::new();
    for enum_val in enum_def.enum_values_definition() {
        diagnostics.extend(db.validate_enum_value(enum_val.clone()));

        // An Enum type must define one or more unique enum values.
        //
        // Return a Unique Value error in case of a duplicate value.
        let value = enum_val.enum_value();
        if let Some(prev_def) = seen.get(&value) {
            let prev_offset = prev_def.loc().offset();
            let prev_node_len = prev_def.loc().node_len();

            let current_offset = enum_val.loc().offset();
            let current_node_len = enum_val.loc().node_len();
            diagnostics.push(ApolloDiagnostic::UniqueDefinition(UniqueDefinition {
                ty: "enum".into(),
                name: value.into(),
                src: db.source_code(prev_def.loc().file_id()),
                original_definition: (prev_offset, prev_node_len).into(),
                redefined_definition: (current_offset, current_node_len).into(),
                help: Some(format!("{value} must only be defined once in this enum.")),
            }));
        } else {
            seen.insert(value, enum_val);
        }
    }

    diagnostics
}

pub(crate) fn validate_enum_value(
    db: &dyn ValidationDatabase,
    enum_val: hir::EnumValueDefinition,
) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = db.validate_directives(
        enum_val.directives().to_vec(),
        hir::DirectiveLocation::EnumValue,
    );

    // (convention) Values in an Enum Definition should be capitalized.
    //
    // Return a Capitalized Value warning if enum value is not capitalized.
    let value = enum_val.enum_value();
    if value.to_uppercase() != value {
        diagnostics.push(ApolloDiagnostic::CapitalizedValue(CapitalizedValue {
            ty: value.into(),
            src: db.source_code(enum_val.loc().file_id()),
            value: (enum_val.loc().offset(), enum_val.loc().node_len()).into(),
        }));
    }

    diagnostics
}
