use std::collections::HashSet;

use crate::{
    diagnostics::{
        RecursiveDefinition, UndefinedDefinition, UniqueDefinition, UnsupportedLocation,
    },
    hir,
    validation::ast_type_definitions,
    ApolloDiagnostic, ValidationDatabase,
};
use apollo_parser::ast;
use miette::SourceSpan;

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
            if let Some(hir_loc) = hir_def.loc() {
                let ast_loc = (file_id, &ast_def).into();
                if *hir_loc == ast_loc {
                    // The HIR node was built from this AST node. This is fine.
                } else {
                    diagnostics.push(ApolloDiagnostic::UniqueDefinition(UniqueDefinition {
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
                    diagnostics.push(ApolloDiagnostic::RecursiveDefinition(RecursiveDefinition {
                        message: format!("{} directive definition cannot reference itself", name),
                        definition: directive.loc().into(),
                        src: db.source_code(directive.loc().file_id()),
                        definition_label: "recursive directive definition".into(),
                    }));
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
        let offset = loc.offset();
        let len = loc.node_len();

        if let Some(directive) = db.find_directive_definition_by_name(name.into()) {
            let directive_def_loc = directive
                .loc
                .map(|loc| SourceSpan::new(loc.offset().into(), loc.node_len().into()));
            let allowed_loc: HashSet<hir::DirectiveLocation> =
                HashSet::from_iter(directive.directive_locations().iter().cloned());
            if !allowed_loc.contains(&dir_loc) {
                diagnostics.push(ApolloDiagnostic::UnsupportedLocation(UnsupportedLocation {
                ty: name.into(),
                dir_loc: dir_loc.clone().into(),
                src: db.source_code(loc.file_id()),
                directive: (offset, len).into(),
                directive_def: directive_def_loc,
                help: Some("the directive must be used in a location that the service has declared support for".into()),
            }))
            }
        } else {
            diagnostics.push(ApolloDiagnostic::UndefinedDefinition(UndefinedDefinition {
                ty: name.into(),
                src: db.source_code(loc.file_id()),
                definition: (offset, len).into(),
            }))
        }
    }
    diagnostics
}
