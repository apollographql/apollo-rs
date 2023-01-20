use std::{collections::HashMap, sync::Arc};

use crate::{
    diagnostics::{UniqueDefinition, UniqueField},
    hir::{self, InputValueDefinition},
    validation::ast_type_definitions,
    ApolloDiagnostic, ValidationDatabase,
};
use apollo_parser::ast;

use super::directive;

pub fn validate(
    db: &dyn ValidationDatabase,
    input_obj: Arc<hir::InputObjectTypeDefinition>,
) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = Vec::new();

    diagnostics.extend(directive::validate_usage(
        db,
        input_obj.directives().to_vec(),
        hir::DirectiveLocation::InputObject,
    ));
    // Input Object Definitions must have unique names.
    //
    // Return a Unique Definition error in case of a duplicate name.
    let hir = db.input_objects();
    for (file_id, ast_def) in ast_type_definitions::<ast::InputObjectTypeDefinition>(db) {
        if let Some(name) = ast_def.name() {
            let name = &*name.text();
            let hir_def = &hir[name];
            let ast_loc = (file_id, &ast_def).into();
            if *hir_def.loc() == ast_loc {
                // The HIR node was built from this AST node. This is fine.
            } else {
                diagnostics.push(ApolloDiagnostic::UniqueDefinition(UniqueDefinition {
                    ty: "input object".into(),
                    name: name.to_owned(),
                    src: db.source_code(hir_def.loc().file_id()),
                    original_definition: hir_def.loc().into(),
                    redefined_definition: ast_loc.into(),
                    help: Some(format!(
                        "`{name}` must only be defined once in this document."
                    )),
                }));
            }
        }
    }

    // Fields in an Input Object Definition must be unique
    //
    // Returns Unique Value error.
    let mut seen: HashMap<&str, &InputValueDefinition> = HashMap::new();

    let input_fields = input_obj.input_fields_definition();

    validate_input_values(
        db,
        input_obj.input_fields_definition.clone(),
        hir::DirectiveLocation::InputFieldDefinition,
    );

    diagnostics
}

pub(crate) fn validate_input_values(
    db: &dyn ValidationDatabase,
    input_values: Arc<Vec<hir::InputValueDefinition>>,
    // directive location depends on parent node location, so we pass this down from parent
    dir_loc: hir::DirectiveLocation,
) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = Vec::new();
    let mut seen: HashMap<&str, &hir::InputValueDefinition> = HashMap::new();

    for input_value in input_values.iter() {
        diagnostics.extend(directive::validate_usage(
            db,
            input_value.directives().to_vec(),
            dir_loc.clone(),
        ));

        let name = input_value.name();
        if let Some(prev_arg) = seen.get(name) {
            let prev_offset = prev_arg.loc().unwrap().offset();
            let prev_node_len = prev_arg.loc().unwrap().node_len();

            let current_offset = input_value.loc().unwrap().offset();
            let current_node_len = input_value.loc().unwrap().node_len();

            diagnostics.push(ApolloDiagnostic::UniqueInputValue(UniqueField {
                field: name.into(),
                src: db.source_code(prev_arg.loc().unwrap().file_id()),
                original_field: (prev_offset, prev_node_len).into(),
                redefined_field: (current_offset, current_node_len).into(),
                help: Some(format!(
                    "`{name}` must only be defined once in input value definition."
                )),
            }));
        } else {
            seen.insert(name, input_value);
        }
    }

    diagnostics
}
