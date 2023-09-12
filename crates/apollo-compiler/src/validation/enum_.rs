use crate::{
    ast,
    diagnostics::{ApolloDiagnostic, DiagnosticData, Label},
    Node, ValidationDatabase,
};
use std::collections::HashMap;

pub fn validate_enum_definitions(db: &dyn ValidationDatabase) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = Vec::new();

    for enum_ in db.ast_types().enums.values() {
        diagnostics.extend(db.validate_enum_definition(enum_.clone()));
    }

    diagnostics
}

pub fn validate_enum_definition(
    db: &dyn ValidationDatabase,
    enum_def: ast::TypeWithExtensions<ast::EnumTypeDefinition>,
) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = super::directive::validate_directives2(
        db,
        enum_def.directives(),
        ast::DirectiveLocation::Enum,
        // enums don't use variables
        Default::default(),
    );

    let mut seen: HashMap<ast::Name, &Node<ast::EnumValueDefinition>> = HashMap::new();
    for enum_val in enum_def.values() {
        diagnostics.extend(validate_enum_value(db, &enum_val));

        // An Enum type must define one or more unique enum values.
        //
        // Return a Unique Definition error in case of a duplicate value.
        if let Some(prev_def) = seen.get(&enum_val.value) {
            let original_definition = *prev_def.location().unwrap();
            let redefined_definition = *enum_val.location().unwrap();
            diagnostics.push(
                ApolloDiagnostic::new(
                    db,
                    redefined_definition.into(),
                    DiagnosticData::UniqueDefinition {
                        ty: "enum value",
                        name: enum_val.value.to_string(),
                        original_definition: original_definition.into(),
                        redefined_definition: redefined_definition.into(),
                    },
                )
                .labels([
                    Label::new(
                        original_definition,
                        format!("previous definition of `{}` here", enum_val.value),
                    ),
                    Label::new(
                        redefined_definition,
                        format!("`{}` redefined here", enum_val.value),
                    ),
                ])
                .help(format!(
                    "{} must only be defined once in this enum.",
                    enum_val.value
                )),
            );
        } else {
            seen.insert(enum_val.value.clone(), &enum_val);
        }
    }

    diagnostics
}

pub(crate) fn validate_enum_value(
    db: &dyn ValidationDatabase,
    enum_val: &Node<ast::EnumValueDefinition>,
) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = super::directive::validate_directives2(
        db,
        enum_val.directives.iter(),
        ast::DirectiveLocation::EnumValue,
        // enum values don't use variables
        Default::default(),
    );

    // (convention) Values in an Enum Definition should be capitalized.
    //
    // Return a Capitalized Value warning if enum value is not capitalized.
    if enum_val.value != enum_val.value.to_uppercase().as_str() {
        let location = *enum_val.location().unwrap();
        diagnostics.push(
            ApolloDiagnostic::new(
                db,
                location.into(),
                DiagnosticData::CapitalizedValue {
                    value: enum_val.value.to_string(),
                },
            )
            .label(Label::new(
                location,
                format!("consider capitalizing {}", enum_val.value),
            )),
        );
    }

    diagnostics
}
