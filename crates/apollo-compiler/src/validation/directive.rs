use crate::{
    diagnostics::{RecursiveDefinition, UniqueDefinition},
    validation::ast_type_definitions,
    ApolloDiagnostic, ValidationDatabase,
};
use apollo_parser::ast;

pub fn check(db: &dyn ValidationDatabase) -> Vec<ApolloDiagnostic> {
    let mut errors = Vec::new();

    // Directive definitions must have unique names.
    //
    // Return a Unique Definition error in case of a duplicate name.
    let hir = db.directive_definitions();
    for (file_id, ast_def) in ast_type_definitions::<ast::DirectiveDefinition>(db) {
        if let Some(name) = ast_def.name() {
            let name = &*name.text();
            let hir_def = &hir[name];
            if let Some(hir_loc) = hir_def.loc() {
                let ast_loc = (file_id, &ast_def).into();
                if *hir_loc == ast_loc {
                    // The HIR node was built from this AST node. This is fine.
                } else {
                    errors.push(ApolloDiagnostic::UniqueDefinition(UniqueDefinition {
                        ty: "directive".into(),
                        name: name.to_owned(),
                        src: db.source_code(hir_loc.file_id()),
                        original_definition: hir_loc.into(),
                        redefined_definition: ast_loc.into(),
                        help: Some(format!(
                            "`{name}` must only be defined once in this document."
                        )),
                    }));
                }
            }
        }
    }

    // A directive definition must not contain the use of a directive which
    // references itself directly.
    //
    // Returns Recursive Definition error.
    for (name, directive_def) in db.directive_definitions().iter() {
        for input_values in directive_def.arguments().input_values() {
            for directive in input_values.directives().iter() {
                let directive_name = directive.name();
                if name == directive_name {
                    errors.push(ApolloDiagnostic::RecursiveDefinition(RecursiveDefinition {
                        message: format!("{} directive definition cannot reference itself", name),
                        definition: directive.loc().into(),
                        src: db.source_code(directive.loc().file_id()),
                        definition_label: "recursive directive definition".into(),
                    }));
                }
            }
        }
    }

    errors
}
