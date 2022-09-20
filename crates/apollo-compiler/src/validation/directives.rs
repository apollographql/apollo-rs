use std::collections::HashMap;

use crate::{
    diagnostics::{RecursiveDefinition, UniqueDefinition},
    hir::DirectiveDefinition,
    ApolloDiagnostic, Document, Validation,
};

pub fn check(db: &dyn Validation) -> Vec<ApolloDiagnostic> {
    let mut errors = Vec::new();

    // Directive definitions must have unique names.
    //
    // Return a Unique Definition error in case of a duplicate name.
    let mut seen: HashMap<&str, &DirectiveDefinition> = HashMap::new();
    for dir_def in db.directive_definitions().iter() {
        let name = dir_def.name();
        if let Some(prev_def) = seen.get(&name) {
            if prev_def.ast_node(db).is_some() && dir_def.ast_node(db).is_some() {
                let prev_offset: usize = prev_def.ast_node(db).unwrap().text_range().start().into();
                let prev_node_len: usize = prev_def.ast_node(db).unwrap().text_range().len().into();

                let current_offset: usize =
                    dir_def.ast_node(db).unwrap().text_range().start().into();
                let current_node_len: usize =
                    dir_def.ast_node(db).unwrap().text_range().len().into();
                errors.push(ApolloDiagnostic::UniqueDefinition(UniqueDefinition {
                    ty: "directive".into(),
                    name: name.into(),
                    src: db.input(),
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
                    let offset = directive.ast_node(db).text_range().start().into();
                    let len: usize = directive.ast_node(db).text_range().len().into();
                    errors.push(ApolloDiagnostic::RecursiveDefinition(RecursiveDefinition {
                        message: format!("{} directive definition cannot reference itself", name),
                        definition: (offset, len).into(),
                        src: db.input(),
                        definition_label: "recursive directive definition".into(),
                    }));
                }
            }
        }
    }

    errors
}
