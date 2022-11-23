use std::collections::HashMap;

use crate::{
    diagnostics::{RecursiveDefinition, UniqueDefinition},
    hir::DirectiveDefinition,
    ApolloDiagnostic, ValidationDatabase,
};

pub fn check(db: &dyn ValidationDatabase) -> Vec<ApolloDiagnostic> {
    let mut errors = Vec::new();

    // Directive definitions must have unique names.
    //
    // Return a Unique Definition error in case of a duplicate name.
    let mut seen: HashMap<&str, &DirectiveDefinition> = HashMap::new();
    for dir_def in db.directive_definitions().iter() {
        let name = dir_def.name();
        if let Some(prev_def) = seen.get(&name) {
            if prev_def.loc.is_some() && dir_def.loc.is_some() {
                let prev_offset: usize = prev_def.loc().unwrap().offset();
                let prev_node_len: usize = prev_def.loc().unwrap().node_len();

                let current_offset: usize = dir_def.loc().unwrap().offset();
                let current_node_len: usize = dir_def.loc().unwrap().node_len();
                errors.push(ApolloDiagnostic::UniqueDefinition(UniqueDefinition {
                    ty: "directive".into(),
                    name: name.into(),
                    src: db.source_code(prev_def.loc().unwrap().file_id()),
                    original_definition: (prev_offset, prev_node_len).into(),
                    redefined_definition: (current_offset, current_node_len).into(),
                    help: Some(format!(
                        "`{name}` must only be defined once in this document."
                    )),
                }));
            }
        } else {
            seen.insert(name, dir_def);
        }
    }

    // A directive definition must not contain the use of a directive which
    // references itself directly.
    //
    // Returns Recursive Definition error.
    for directive_def in db.directive_definitions().iter() {
        let name = directive_def.name();
        for input_values in directive_def.arguments().input_values() {
            for directive in input_values.directives().iter() {
                let directive_name = directive.name();
                if name == directive_name {
                    let offset = directive.loc().offset();
                    let len: usize = directive.loc().node_len();
                    errors.push(ApolloDiagnostic::RecursiveDefinition(RecursiveDefinition {
                        message: format!("{} directive definition cannot reference itself", name),
                        definition: (offset, len).into(),
                        src: db.source_code(directive.loc().file_id()),
                        definition_label: "recursive directive definition".into(),
                    }));
                }
            }
        }
    }

    errors
}
