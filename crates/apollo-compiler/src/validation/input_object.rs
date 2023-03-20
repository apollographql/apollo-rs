use std::{collections::HashMap, sync::Arc};

use crate::{
    diagnostics::{ApolloDiagnostic, DiagnosticData, Label},
    hir, ValidationDatabase,
};

pub fn validate_input_object_definitions(db: &dyn ValidationDatabase) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = Vec::new();

    let defs = &db.type_system_definitions().input_objects;
    for def in defs.values() {
        diagnostics.extend(db.validate_input_object_definition(def.clone()));
    }

    diagnostics
}

pub fn validate_input_object_definition(
    db: &dyn ValidationDatabase,
    input_obj: Arc<hir::InputObjectTypeDefinition>,
) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = db.validate_directives(
        input_obj.directives().cloned().collect(),
        hir::DirectiveLocation::InputObject,
    );

    // Fields in an Input Object Definition must be unique
    //
    // Returns Unique Definition error.
    diagnostics.extend(db.validate_input_values(
        input_obj.input_fields_definition.clone(),
        hir::DirectiveLocation::InputFieldDefinition,
    ));

    diagnostics
}

pub fn validate_input_values(
    db: &dyn ValidationDatabase,
    input_values: Arc<Vec<hir::InputValueDefinition>>,
    // directive location depends on parent node location, so we pass this down from parent
    dir_loc: hir::DirectiveLocation,
) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = Vec::new();
    let mut seen: HashMap<&str, &hir::InputValueDefinition> = HashMap::new();

    for input_value in input_values.iter() {
        diagnostics.extend(db.validate_directives(input_value.directives().to_vec(), dir_loc));

        let name = input_value.name();
        if let Some(prev_arg) = seen.get(name) {
            if let (Some(original_value), Some(redefined_value)) =
                (prev_arg.loc(), input_value.loc())
            {
                diagnostics.push(
                    ApolloDiagnostic::new(
                        db,
                        original_value.into(),
                        DiagnosticData::UniqueInputValue {
                            name: name.into(),
                            original_value: original_value.into(),
                            redefined_value: redefined_value.into(),
                        },
                    )
                    .labels([
                        Label::new(
                            original_value,
                            format!("previous definition of `{name}` here"),
                        ),
                        Label::new(redefined_value, format!("`{name}` redefined here")),
                    ])
                    .help(format!(
                        "`{name}` field must only be defined once in this input object definition."
                    )),
                );
            }
        } else {
            seen.insert(name, input_value);
        }
    }

    diagnostics
}
