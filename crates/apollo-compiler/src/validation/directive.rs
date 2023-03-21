use std::collections::HashSet;

use crate::{
    diagnostics::{ApolloDiagnostic, DiagnosticData, Label},
    hir,
    validation::ValidationSet,
    ValidationDatabase,
};

pub fn validate_directive_definitions(db: &dyn ValidationDatabase) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = Vec::new();

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

    let mut seen_dirs = HashSet::<ValidationSet>::new();

    for dir in dirs {
        diagnostics.extend(db.validate_arguments(dir.arguments().to_vec()));

        let name = dir.name();
        let loc = dir.loc();
        let directive_definition = db.find_directive_definition_by_name(name.into());

        let duplicate = ValidationSet {
            name: name.to_string(),
            loc: Some(loc),
        };
        if let Some(original) = seen_dirs.get(&duplicate) {
            let is_repeatable = directive_definition
                .as_ref()
                .map(|def| def.repeatable())
                // Assume unknown directives are repeatable to avoid producing confusing diagnostics
                .unwrap_or(true);

            if !is_repeatable {
                // original loc must be Some
                let loc = original.loc.expect("undefined original directive location");
                diagnostics.push(
                    ApolloDiagnostic::new(
                        db,
                        loc.into(),
                        DiagnosticData::UniqueDirective {
                            name: name.to_string(),
                            original_call: loc.into(),
                            conflicting_call: loc.into(),
                        },
                    )
                    .label(Label::new(
                        loc,
                        format!("directive {name} first called here"),
                    ))
                    .label(Label::new(
                        loc,
                        format!("directive {name} called again here"),
                    )),
                );
            }
        } else {
            seen_dirs.insert(duplicate);
        }

        if let Some(directive_definition) = directive_definition {
            let allowed_loc: HashSet<hir::DirectiveLocation> =
                HashSet::from_iter(directive_definition.directive_locations().iter().cloned());
            if !allowed_loc.contains(&dir_loc) {
                let mut diag = ApolloDiagnostic::new(
                        db,
                        loc.into(),
                        DiagnosticData::UnsupportedLocation {
                            name: name.into(),
                            dir_loc,
                            directive_def: directive_definition.loc.map(|loc| loc.into()),
                        },
                )
                    .label(Label::new(loc, format!("{dir_loc} is not a valid location")))
                    .help("the directive must be used in a location that the service has declared support for");
                if let Some(directive_def_loc) = directive_definition.loc {
                    diag = diag.label(Label::new(
                        directive_def_loc,
                        format!("consider adding {dir_loc} directive location here"),
                    ));
                }
                diagnostics.push(diag)
            }

            for arg in dir.arguments() {
                let exists = directive_definition
                    .arguments()
                    .input_values()
                    .iter()
                    .any(|arg_def| arg.name() == arg_def.name());

                if !exists {
                    let mut diagnostic = ApolloDiagnostic::new(
                        db,
                        arg.loc.into(),
                        DiagnosticData::UndefinedArgument {
                            name: arg.name().into(),
                        },
                    )
                    .label(Label::new(arg.loc, "argument by this name not found"));
                    if let Some(loc) = directive_definition.loc {
                        diagnostic = diagnostic.label(Label::new(loc, "directive declared here"));
                    }

                    diagnostics.push(diagnostic);
                }
            }

            for arg_def in directive_definition.arguments().input_values() {
                let arg_value = dir
                    .arguments()
                    .iter()
                    .find(|value| value.name() == arg_def.name());
                let is_null = match arg_value {
                    None => true,
                    // Prevents explicitly providing `requiredArg: null`,
                    // but you can still indirectly do the wrong thing by typing `requiredArg: $mayBeNull`
                    // and it won't raise a validation error at this stage.
                    Some(value) => value.value() == &hir::Value::Null,
                };

                if arg_def.is_required() && is_null {
                    let mut diagnostic = ApolloDiagnostic::new(
                        db,
                        dir.loc.into(),
                        DiagnosticData::RequiredArgument {
                            name: arg_def.name().into(),
                        },
                    );
                    diagnostic = diagnostic.label(Label::new(
                        dir.loc,
                        format!("missing value for argument `{}`", arg_def.name()),
                    ));
                    if let Some(loc) = arg_def.loc {
                        diagnostic = diagnostic.label(Label::new(loc, "argument defined here"));
                    }

                    diagnostics.push(diagnostic);
                }
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
