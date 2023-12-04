use crate::diagnostics::{ApolloDiagnostic, DiagnosticData, Label};
use crate::validation::{NodeLocation, RecursionGuard, RecursionStack};
use crate::{ast, schema, Node, ValidationDatabase};
use std::collections::{HashMap, HashSet};

use super::CycleError;

/// This struct just groups functions that are used to find self-referential directives.
/// The way to use it is to call `FindRecursiveDirective::check`.
struct FindRecursiveDirective<'s> {
    schema: &'s schema::Schema,
}

impl FindRecursiveDirective<'_> {
    fn type_definition(
        &self,
        seen: &mut RecursionGuard<'_>,
        def: &schema::ExtendedType,
    ) -> Result<(), CycleError<ast::Directive>> {
        match def {
            schema::ExtendedType::Scalar(scalar_type_definition) => {
                self.directives(seen, &scalar_type_definition.directives)?;
            }
            schema::ExtendedType::Object(object_type_definition) => {
                self.directives(seen, &object_type_definition.directives)?;
            }
            schema::ExtendedType::Interface(interface_type_definition) => {
                self.directives(seen, &interface_type_definition.directives)?;
            }
            schema::ExtendedType::Union(union_type_definition) => {
                self.directives(seen, &union_type_definition.directives)?;
            }
            schema::ExtendedType::Enum(enum_type_definition) => {
                self.directives(seen, &enum_type_definition.directives)?;
                for enum_value in enum_type_definition.values.values() {
                    self.enum_value(seen, enum_value)?;
                }
            }
            schema::ExtendedType::InputObject(input_type_definition) => {
                self.directives(seen, &input_type_definition.directives)?;
                for input_value in input_type_definition.fields.values() {
                    self.input_value(seen, input_value)?;
                }
            }
        }

        Ok(())
    }

    fn input_value(
        &self,
        seen: &mut RecursionGuard<'_>,
        input_value: &Node<ast::InputValueDefinition>,
    ) -> Result<(), CycleError<ast::Directive>> {
        for directive in &input_value.directives {
            self.directive(seen, directive)?;
        }

        let type_name = input_value.ty.inner_named_type();
        if let Some(type_def) = self.schema.types.get(type_name) {
            self.type_definition(seen, type_def)?;
        }

        Ok(())
    }

    fn enum_value(
        &self,
        seen: &mut RecursionGuard<'_>,
        enum_value: &Node<ast::EnumValueDefinition>,
    ) -> Result<(), CycleError<ast::Directive>> {
        for directive in &enum_value.directives {
            self.directive(seen, directive)?;
        }

        Ok(())
    }

    fn directives(
        &self,
        seen: &mut RecursionGuard<'_>,
        directives: &[schema::Component<ast::Directive>],
    ) -> Result<(), CycleError<ast::Directive>> {
        for directive in directives {
            self.directive(seen, directive)?;
        }
        Ok(())
    }

    fn directive(
        &self,
        seen: &mut RecursionGuard<'_>,
        directive: &Node<ast::Directive>,
    ) -> Result<(), CycleError<ast::Directive>> {
        if !seen.contains(&directive.name) {
            if let Some(def) = self.schema.directive_definitions.get(&directive.name) {
                self.directive_definition(seen.push(&directive.name)?, def)
                    .map_err(|error| error.trace(directive))?;
            }
        } else if seen.first() == Some(&directive.name) {
            // Only report an error & bail out early if this is the *initial* directive.
            // This prevents raising confusing errors when a directive `@b` which is not
            // self-referential uses a directive `@a` that *is*. The error with `@a` should
            // only be reported on its definition, not on `@b`'s.
            return Err(CycleError::Recursed(vec![directive.clone()]));
        }

        Ok(())
    }

    fn directive_definition(
        &self,
        mut seen: RecursionGuard<'_>,
        def: &Node<ast::DirectiveDefinition>,
    ) -> Result<(), CycleError<ast::Directive>> {
        for input_value in &def.arguments {
            self.input_value(&mut seen, input_value)?;
        }

        Ok(())
    }

    fn check(
        schema: &schema::Schema,
        directive_def: &Node<ast::DirectiveDefinition>,
    ) -> Result<(), CycleError<ast::Directive>> {
        let mut recursion_stack = RecursionStack::with_root(directive_def.name.clone());
        FindRecursiveDirective { schema }
            .directive_definition(recursion_stack.guard(), directive_def)
    }
}

pub(crate) fn validate_directive_definition(
    db: &dyn ValidationDatabase,
    def: Node<ast::DirectiveDefinition>,
) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = vec![];

    diagnostics.extend(super::input_object::validate_argument_definitions(
        db,
        &def.arguments,
        ast::DirectiveLocation::ArgumentDefinition,
    ));

    // A directive definition must not contain the use of a directive which
    // references itself directly.
    //
    // Returns Recursive Definition error.
    match FindRecursiveDirective::check(&db.schema(), &def) {
        Ok(_) => {}
        Err(CycleError::Recursed(trace)) => {
            let definition_location = def.location();
            let head_location = NodeLocation::recompose(def.location(), def.name.location());
            let mut diagnostic = ApolloDiagnostic::new(
                db,
                definition_location,
                DiagnosticData::RecursiveDirectiveDefinition {
                    name: def.name.to_string(),
                },
            )
            .label(Label::new(head_location, "recursive directive definition"));

            if let Some((cyclical_application, path)) = trace.split_first() {
                let mut prev_name = &def.name;
                for directive_application in path.iter().rev() {
                    diagnostic = diagnostic.label(Label::new(
                        directive_application.location(),
                        format!(
                            "`{}` references `{}` here...",
                            prev_name, directive_application.name
                        ),
                    ));
                    prev_name = &directive_application.name;
                }

                diagnostic = diagnostic.label(Label::new(
                    cyclical_application.location(),
                    format!(
                        "`{}` circularly references `{}` here",
                        prev_name, cyclical_application.name
                    ),
                ));
            }

            diagnostics.push(diagnostic);
        }
        Err(CycleError::Limit(_)) => {
            let diagnostic = ApolloDiagnostic::new(
                db,
                def.location(),
                DiagnosticData::DeeplyNestedType {
                    name: def.name.to_string(),
                },
            )
            .label(Label::new(
                def.location(),
                "directive references a very long chain of directives",
            ));

            diagnostics.push(diagnostic);
        }
    }

    diagnostics
}

pub(crate) fn validate_directive_definitions(db: &dyn ValidationDatabase) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = Vec::new();

    for file_id in db.type_definition_files() {
        for def in &db.ast(file_id).definitions {
            if let ast::Definition::DirectiveDefinition(directive_definition) = def {
                diagnostics.extend(db.validate_directive_definition(directive_definition.clone()));
            }
        }
    }

    diagnostics
}

// TODO(@goto-bus-stop) This is a big function: should probably not be generic over the iterator
// type
pub(crate) fn validate_directives<'dir>(
    db: &dyn ValidationDatabase,
    dirs: impl Iterator<Item = &'dir Node<ast::Directive>>,
    dir_loc: ast::DirectiveLocation,
    var_defs: &[Node<ast::VariableDefinition>],
) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = Vec::new();

    let mut seen_directives = HashMap::<_, Option<NodeLocation>>::new();

    let schema = db.schema();
    for dir in dirs {
        diagnostics.extend(super::argument::validate_arguments(db, &dir.arguments));

        let name = &dir.name;
        let loc = dir.location();
        let directive_definition = schema.directive_definitions.get(name);

        if let Some(&original_loc) = seen_directives.get(name) {
            let is_repeatable = directive_definition
                .map(|def| def.repeatable)
                // Assume unknown directives are repeatable to avoid producing confusing diagnostics
                .unwrap_or(true);

            if !is_repeatable {
                diagnostics.push(
                    ApolloDiagnostic::new(
                        db,
                        loc,
                        DiagnosticData::UniqueDirective {
                            name: name.to_string(),
                            original_call: original_loc,
                            conflicting_call: loc,
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
            let loc = NodeLocation::recompose(dir.location(), dir.name.location());
            seen_directives.insert(&dir.name, loc);
        }

        if let Some(directive_definition) = directive_definition {
            let allowed_loc: HashSet<ast::DirectiveLocation> =
                HashSet::from_iter(directive_definition.locations.iter().cloned());
            if !allowed_loc.contains(&dir_loc) {
                let mut diag = ApolloDiagnostic::new(
                        db,
                        loc,
                        DiagnosticData::UnsupportedLocation {
                            name: name.to_string(),
                            dir_loc,
                            directive_def: directive_definition.location(),
                        },
                )
                    .label(Label::new(loc, format!("{dir_loc} is not a valid location")))
                    .help("the directive must be used in a location that the service has declared support for");
                if !directive_definition.is_built_in() {
                    diag = diag.label(Label::new(
                        directive_definition.location(),
                        format!("consider adding {dir_loc} directive location here"),
                    ));
                }
                diagnostics.push(diag)
            }

            for argument in &dir.arguments {
                let input_value = directive_definition
                    .arguments
                    .iter()
                    .find(|val| val.name == argument.name)
                    .cloned();

                // @b(a: true)
                if let Some(input_value) = input_value {
                    // TODO(@goto-bus-stop) do we really need value validation and variable
                    // validation separately?
                    if let Some(diag) = super::variable::validate_variable_usage(
                        db,
                        input_value.clone(),
                        var_defs,
                        argument,
                    )
                    .err()
                    {
                        diagnostics.push(diag)
                    } else {
                        let type_diags =
                            super::value::validate_values(db, &input_value.ty, argument, var_defs);

                        diagnostics.extend(type_diags);
                    }
                } else {
                    diagnostics.push(
                        ApolloDiagnostic::new(
                            db,
                            argument.location(),
                            DiagnosticData::UndefinedArgument {
                                name: argument.name.to_string(),
                            },
                        )
                        .label(Label::new(
                            argument.name.location(),
                            "argument by this name not found",
                        ))
                        .label(Label::new(loc, "directive declared here")),
                    );
                }
            }
            for arg_def in &directive_definition.arguments {
                let arg_value = dir
                    .arguments
                    .iter()
                    .find_map(|arg| (arg.name == arg_def.name).then_some(&arg.value));
                let is_null = match arg_value {
                    None => true,
                    // Prevents explicitly providing `requiredArg: null`,
                    // but you can still indirectly do the wrong thing by typing `requiredArg: $mayBeNull`
                    // and it won't raise a validation error at this stage.
                    Some(value) => value.is_null(),
                };

                if arg_def.is_required() && is_null && arg_def.default_value.is_none() {
                    let mut diagnostic = ApolloDiagnostic::new(
                        db,
                        dir.location(),
                        DiagnosticData::RequiredArgument {
                            name: arg_def.name.to_string(),
                        },
                    );
                    diagnostic = diagnostic.label(Label::new(
                        dir.location(),
                        format!("missing value for argument `{}`", arg_def.name),
                    ));
                    let loc = arg_def.location();
                    diagnostic = diagnostic.label(Label::new(loc, "argument defined here"));

                    diagnostics.push(diagnostic);
                }
            }
        } else {
            diagnostics.push(
                ApolloDiagnostic::new(
                    db,
                    loc,
                    DiagnosticData::UndefinedDirective {
                        name: name.to_string(),
                    },
                )
                .label(Label::new(loc, "directive not defined")),
            )
        }
    }

    diagnostics
}
