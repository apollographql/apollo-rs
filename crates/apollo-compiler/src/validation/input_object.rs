use std::{collections::HashMap, sync::Arc};

use crate::{
    diagnostics::{ApolloDiagnostic, DiagnosticData, Label},
    hir,
    validation::ast_type_definitions,
    ValidationDatabase,
};
use apollo_parser::ast;

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
        input_obj.directives().to_vec(),
        hir::DirectiveLocation::InputObject,
    );
    // Input Object Definitions must have unique names.
    //
    // Return a Unique Definition error in case of a duplicate name.
    let hir = db.input_objects();
    for (file_id, ast_def) in ast_type_definitions::<ast::InputObjectTypeDefinition>(db) {
        if let Some(name) = ast_def.name() {
            let name = &*name.text();
            let hir_def = &hir[name];
            let original_definition = hir_def.loc();
            let redefined_definition = (file_id, &ast_def).into();
            if original_definition == redefined_definition {
                // The HIR node was built from this AST node. This is fine.
            } else {
                diagnostics.push(
                    ApolloDiagnostic::new(
                        db,
                        original_definition.into(),
                        DiagnosticData::UniqueDefinition {
                            ty: "input object",
                            name: name.to_owned(),
                            original_definition: original_definition.into(),
                            redefined_definition: redefined_definition.into(),
                        },
                    )
                    .help(format!(
                        "`{name}` must only be defined once in this document."
                    ))
                    .labels([
                        Label::new(
                            original_definition,
                            format!("previous definition of `{name}` here"),
                        ),
                        Label::new(redefined_definition, format!("`{name}` redefined here")),
                    ]),
                );
            }
        }
    }

    // Fields in an Input Object Definition must be unique
    //
    // Returns Unique Value error.
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
