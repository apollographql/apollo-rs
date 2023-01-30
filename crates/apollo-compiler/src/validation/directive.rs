use std::collections::HashSet;

use crate::{
    diagnostics::{ApolloDiagnostic, DiagnosticData, Label},
    hir,
    validation::ast_type_definitions,
    ValidationDatabase,
};
use apollo_parser::ast;

pub fn validate_directive_definitions(db: &dyn ValidationDatabase) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = Vec::new();

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
                if original_definition == redefined_definition {
                    // The HIR node was built from this AST node. This is fine.
                } else {
                    diagnostics.push(
                        ApolloDiagnostic::new(
                            db,
                            original_definition.into(),
                            DiagnosticData::UniqueDefinition {
                                ty: "directive",
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
                    diagnostics.push(
                        ApolloDiagnostic::new(
                            db,
                            directive.loc().into(),
                            DiagnosticData::RecursiveDefinition { name: name.clone() },
                        )
                        .label(Label::new(
                            directive.loc(),
                            "recursive directive definition",
                        )),
                    );
                }
            }
        }

        // Validate directive definitions' arguments
        diagnostics.extend(db.validate_arguments_definition(
            directive_def.arguments.clone(),
            hir::DirectiveLocation::ArgumentDefinition,
        ));
    }

    diagnostics
}

pub fn validate_directives(
    db: &dyn ValidationDatabase,
    dirs: Vec<hir::Directive>,
    dir_loc: hir::DirectiveLocation,
) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = Vec::new();
    for dir in dirs {
        diagnostics.extend(db.validate_arguments(dir.arguments().to_vec()));

        let name = dir.name();
        let loc = dir.loc();

        if let Some(directive) = db.find_directive_definition_by_name(name.into()) {
            let allowed_loc: HashSet<hir::DirectiveLocation> =
                HashSet::from_iter(directive.directive_locations().iter().cloned());
            if !allowed_loc.contains(&dir_loc) {
                let mut diag = ApolloDiagnostic::new(
                        db,
                        loc.into(),
                        DiagnosticData::UnsupportedLocation {
                            name: name.into(),
                            dir_loc,
                            directive_def: directive.loc.map(|loc| loc.into()),
                        },
                )
                    .label(Label::new(loc, format!("{dir_loc} is not a valid location")))
                    .help("the directive must be used in a location that the service has declared support for");
                if let Some(directive_def_loc) = directive.loc {
                    diag = diag.label(Label::new(
                        directive_def_loc,
                        format!("consider adding {dir_loc} directive location here"),
                    ));
                }
                diagnostics.push(diag)
            }
        } else {
            diagnostics.push(
                ApolloDiagnostic::new(
                    db,
                    loc.into(),
                    DiagnosticData::UndefinedDefinition { name: name.into() },
                )
                .label(Label::new(loc, "not found in this scope")),
            )
        }
    }
    diagnostics
}
