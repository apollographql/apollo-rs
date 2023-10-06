use std::{collections::HashMap, sync::Arc};

use crate::{
    diagnostics::{ApolloDiagnostic, DiagnosticData, Label},
    hir::{self, EnumValueDefinition},
    ValidationDatabase,
};

pub fn validate_enum_definitions(db: &dyn ValidationDatabase) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = Vec::new();

    let defs = &db.type_system_definitions().enums;
    for def in defs.values() {
        diagnostics.extend(db.validate_enum_definition(def.clone()));
    }

    diagnostics
}

fn iter_with_extensions<'a, Item, Ext>(
    base: &'a [Item],
    extensions: &'a [Arc<Ext>],
    method: impl Fn(&'a Ext) -> &'a [Item],
) -> impl Iterator<Item = &'a Item> {
    base.iter()
        .chain(extensions.iter().flat_map(move |ext| method(ext).iter()))
}

pub fn validate_enum_definition(
    db: &dyn ValidationDatabase,
    enum_def: Arc<hir::EnumTypeDefinition>,
) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = db.validate_directives(
        enum_def.directives().cloned().collect(),
        hir::DirectiveLocation::Enum,
        // enums don't use variables
        Arc::new(Vec::new()),
    );

    let enum_values = iter_with_extensions(
        enum_def.self_values(),
        enum_def.extensions(),
        hir::EnumTypeExtension::values,
    );

    let mut seen: HashMap<&str, &EnumValueDefinition> = HashMap::new();
    for enum_val in enum_values {
        diagnostics.extend(db.validate_enum_value(enum_val.clone()));

        // An Enum type must define one or more unique enum values.
        //
        // Return a Unique Definition error in case of a duplicate value.
        let value = enum_val.enum_value();
        if let Some(prev_def) = seen.get(&value) {
            let original_definition = prev_def.loc();
            let redefined_definition = enum_val.loc();
            diagnostics.push(
                ApolloDiagnostic::new(
                    db,
                    redefined_definition.into(),
                    DiagnosticData::UniqueEnumValue {
                        name: value.into(),
                        coordinate: enum_def.name().to_string(),
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
        // enum values don't use variables
        Arc::new(Vec::new()),
    );

    // (convention) Values in an Enum Definition should be capitalized.
    //
    // Return a Capitalized Value warning if enum value is not capitalized.
    let value = enum_val.enum_value();
    if value.to_uppercase() != value {
        diagnostics.push(
            ApolloDiagnostic::new(
                db,
                enum_val.loc().into(),
                DiagnosticData::CapitalizedValue {
                    value: value.into(),
                },
            )
            .label(Label::new(
                enum_val.loc(),
                format!("consider capitalizing {value}"),
            )),
        );
    }

    diagnostics
}
