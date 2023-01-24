use std::collections::HashMap;

use crate::{
    diagnostics::UniqueArgument,
    hir::{self, DirectiveLocation},
    validation::{input_object, ValidationDatabase},
    ApolloDiagnostic,
};

pub fn validate_arguments(
    db: &dyn ValidationDatabase,
    args: Vec<hir::Argument>,
) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = Vec::new();
    let mut seen: HashMap<&str, &hir::Argument> = HashMap::new();

    for arg in &args {
        let name = arg.name();
        if let Some(prev_arg) = seen.get(name) {
            let prev_offset = prev_arg.loc().offset();
            let prev_node_len = prev_arg.loc().node_len();

            let current_offset = arg.loc().offset();
            let current_node_len = arg.loc().node_len();

            diagnostics.push(ApolloDiagnostic::UniqueArgument(UniqueArgument {
                name: name.into(),
                src: db.source_code(prev_arg.loc().file_id()),
                original_definition: (prev_offset, prev_node_len).into(),
                redefined_definition: (current_offset, current_node_len).into(),
                help: Some(format!("`{name}` argument must only be provided once.")),
            }));
        } else {
            seen.insert(name, arg);
        }
    }

    diagnostics
}

pub fn validate_arguments_definition(
    db: &dyn ValidationDatabase,
    args_def: hir::ArgumentsDefinition,
    dir_loc: DirectiveLocation,
) -> Vec<ApolloDiagnostic> {
    input_object::validate_input_values(db, args_def.input_values, dir_loc)
}
