use crate::{
    diagnostics::{Diagnostic2, DiagnosticData, Label},
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
            if let Some(original_definition) = hir_def.loc() {
                let redefined_definition = (file_id, &ast_def).into();
                if *original_definition == redefined_definition {
                    // The HIR node was built from this AST node. This is fine.
                } else {
                    errors.push(ApolloDiagnostic::Diagnostic2(
                        Diagnostic2::new(
                            *original_definition,
                            DiagnosticData::UniqueDefinition {
                                ty: "directive",
                                name: name.to_owned(),
                                original_definition: *original_definition,
                                redefined_definition,
                            },
                        )
                        .help(format!(
                            "`{name}` must only be defined once in this document."
                        ))
                        .labels([
                            Label::new(
                                *original_definition,
                                format!("previous definition of `{}` here", name),
                            ),
                            Label::new(redefined_definition, format!("`{}` redefined here", name)),
                        ]),
                    ));
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
                    errors.push(ApolloDiagnostic::Diagnostic2(
                        Diagnostic2::new(
                            *directive.loc(),
                            DiagnosticData::RecursiveDefinition { name: name.clone() },
                        )
                        .label(Label::new(
                            *directive.loc(),
                            "recursive directive definition",
                        )),
                    ));
                }
            }
        }
    }

    errors
}
