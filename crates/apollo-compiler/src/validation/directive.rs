use std::{collections::HashSet, sync::Arc};

use crate::{
    diagnostics::{ApolloDiagnostic, DiagnosticData, Label},
    hir,
    validation::{RecursionStack, ValidationSet},
    ValidationDatabase,
};

/// This struct just groups functions that are used to find self-referential directives.
/// The way to use it is to call `FindRecursiveDirective::check`.
struct FindRecursiveDirective<'a> {
    db: &'a dyn ValidationDatabase,
}

impl FindRecursiveDirective<'_> {
    fn type_definition(
        &self,
        seen: &mut RecursionStack<'_>,
        def: &hir::TypeDefinition,
    ) -> Result<(), hir::Directive> {
        for directive in def.directives() {
            self.directive(seen, directive)?;
        }

        match def {
            hir::TypeDefinition::InputObjectTypeDefinition(input_type_definition) => {
                for input_value in input_type_definition.fields() {
                    self.input_value(seen, input_value)?;
                }
            }
            hir::TypeDefinition::EnumTypeDefinition(enum_type_definition) => {
                for enum_value in enum_type_definition.values() {
                    self.enum_value(seen, enum_value)?;
                }
            }
            _ => (),
        }

        Ok(())
    }

    fn input_value(
        &self,
        seen: &mut RecursionStack<'_>,
        input_value: &hir::InputValueDefinition,
    ) -> Result<(), hir::Directive> {
        for directive in input_value.directives() {
            self.directive(seen, directive)?;
        }
        if let Some(type_def) = input_value.ty().type_def(self.db.upcast()) {
            self.type_definition(seen, &type_def)?;
        }

        Ok(())
    }

    fn enum_value(
        &self,
        seen: &mut RecursionStack<'_>,
        enum_value: &hir::EnumValueDefinition,
    ) -> Result<(), hir::Directive> {
        for directive in enum_value.directives() {
            self.directive(seen, directive)?;
        }

        Ok(())
    }

    fn directive(
        &self,
        seen: &mut RecursionStack<'_>,
        directive: &hir::Directive,
    ) -> Result<(), hir::Directive> {
        if !seen.contains(directive.name()) {
            if let Some(def) = directive.directive(self.db.upcast()) {
                self.directive_definition(seen.push(directive.name().to_string()), &def)?;
            }
        } else if seen.first() == Some(directive.name()) {
            // Only report an error & bail out early if this is the *initial* directive.
            // This prevents raising confusing errors when a directive `@b` which is not
            // self-referential uses a directive `@a` that *is*. The error with `@a` should
            // only be reported on its definition, not on `@b`'s.
            return Err(directive.clone());
        }

        Ok(())
    }

    fn directive_definition(
        &self,
        mut seen: RecursionStack<'_>,
        def: &hir::DirectiveDefinition,
    ) -> Result<(), hir::Directive> {
        let mut guard = seen.push(def.name().to_string());
        for input_value in def.arguments().input_values() {
            self.input_value(&mut guard, input_value)?;
        }

        Ok(())
    }

    fn check(
        db: &dyn ValidationDatabase,
        directive_def: &hir::DirectiveDefinition,
    ) -> Result<(), hir::Directive> {
        FindRecursiveDirective { db }
            .directive_definition(RecursionStack(&mut vec![]), directive_def)
    }
}

pub fn validate_directive_definitions(db: &dyn ValidationDatabase) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = Vec::new();

    // A directive definition must not contain the use of a directive which
    // references itself directly.
    //
    // Returns Recursive Definition error.
    for (name, directive_def) in db.directive_definitions().iter() {
        if let Err(directive) = FindRecursiveDirective::check(db, directive_def) {
            diagnostics.push(
                ApolloDiagnostic::new(
                    db,
                    directive_def.loc().into(),
                    DiagnosticData::RecursiveDirectiveDefinition { name: name.clone() },
                )
                .label(Label::new(
                    directive_def.head_loc(),
                    "recursive directive definition",
                ))
                .label(Label::new(directive.loc(), "refers to itself here")),
            );
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
    var_defs: Arc<Vec<hir::VariableDefinition>>,
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
                let original_loc = original.loc.expect("undefined original directive location");
                diagnostics.push(
                    ApolloDiagnostic::new(
                        db,
                        loc.into(),
                        DiagnosticData::UniqueDirective {
                            name: name.to_string(),
                            original_call: original_loc.into(),
                            conflicting_call: loc.into(),
                        },
                    )
                    .label(Label::new(
                        original_loc,
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
                            directive_def: directive_definition.loc.into(),
                        },
                )
                    .label(Label::new(loc, format!("{dir_loc} is not a valid location")))
                    .help("the directive must be used in a location that the service has declared support for");
                if !directive_definition.is_built_in() {
                    diag = diag.label(Label::new(
                        directive_definition.loc,
                        format!("consider adding {dir_loc} directive location here"),
                    ));
                }
                diagnostics.push(diag)
            }

            for arg in dir.arguments() {
                let input_val = directive_definition
                    .arguments()
                    .input_values()
                    .iter()
                    .find(|val| arg.name() == val.name())
                    .cloned();

                // @b(a: true)
                if let Some(input_val) = input_val {
                    if let Some(diag) = db
                        .validate_variable_usage(input_val.clone(), var_defs.clone(), arg.clone())
                        .err()
                    {
                        diagnostics.push(diag)
                    } else {
                        let ty = input_val.ty().type_def(db.upcast());
                        match arg.value() {
                            hir::Value::Variable(_) => todo!(),
                            hir::Value::Int(_) => todo!(),
                            hir::Value::Float(_) => todo!(),
                            hir::Value::String(_) => todo!(),
                            hir::Value::Boolean(_) => todo!(),
                            hir::Value::Null => todo!(),
                            hir::Value::Enum(_) => todo!(),
                            hir::Value::List(_) => todo!(),
                            hir::Value::Object(_) => todo!(), //
                        }
                    }
                } else {
                    diagnostics.push(
                        ApolloDiagnostic::new(
                            db,
                            arg.loc.into(),
                            DiagnosticData::UndefinedArgument {
                                name: arg.name().into(),
                            },
                        )
                        .label(Label::new(arg.loc, "argument by this name not found"))
                        .label(Label::new(loc, "directive declared here")),
                    );
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
