use crate::diagnostics::{ApolloDiagnostic, DiagnosticData, Label};
use crate::validation::operation::OperationValidationConfig;
use crate::validation::{CycleError, FileId, NodeLocation, RecursionGuard, RecursionStack};
use crate::{ast, schema, Node, ValidationDatabase};
use std::collections::{HashMap, HashSet};

/// Given a type definition, find all the type names that can be used for fragment spreading.
///
/// Spec: https://spec.graphql.org/October2021/#GetPossibleTypes()
pub(crate) fn get_possible_types<'a>(
    schema: &'a schema::Schema,
    type_name: &'a ast::Name,
) -> HashSet<&'a ast::NamedType> {
    match schema.types.get(type_name) {
        // 1. If `type` is an object type, return a set containing `type`.
        Some(schema::ExtendedType::Object(_)) => std::iter::once(type_name).collect(),
        // 2. If `type` is an interface type, return the set of types implementing `type`.
        Some(schema::ExtendedType::Interface(_)) => {
            // TODO(@goto-bus-stop): use db.implementers_map()
            schema
                .types
                .iter()
                .filter_map(|(name, ty)| {
                    let implements = match ty {
                        schema::ExtendedType::Object(object) => &object.implements_interfaces,
                        _ => return None,
                    };

                    if implements.contains(type_name) {
                        Some(name)
                    } else {
                        None
                    }
                })
                .collect()
        }
        // 3. If `type` is a union type, return the set of possible types of `type`.
        Some(schema::ExtendedType::Union(union_)) => union_
            .members
            .iter()
            .map(|component| &component.name)
            .collect(),
        _ => Default::default(),
    }
}

fn validate_fragment_spread_type(
    db: &dyn ValidationDatabase,
    file_id: FileId,
    against_type: &ast::NamedType,
    type_condition: &ast::NamedType,
    selection: &ast::Selection,
) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = Vec::new();
    let schema = db.schema();

    // Another diagnostic will be raised if the type condition was wrong.
    // We reduce noise by silencing other issues with the fragment.
    if !schema.types.contains_key(type_condition) {
        return diagnostics;
    };

    let Some(against_type_definition) = schema.types.get(against_type) else {
        // We cannot check anything if the parent type is unknown.
        return diagnostics;
    };

    let concrete_parent_types = get_possible_types(&schema, against_type);
    let concrete_condition_types = get_possible_types(&schema, type_condition);

    let mut applicable_types = concrete_parent_types.intersection(&concrete_condition_types);
    if applicable_types.next().is_none() {
        // Report specific errors for the different kinds of fragments.
        let diagnostic = match selection {
            ast::Selection::Field(_) => unreachable!(),
            ast::Selection::FragmentSpread(spread) => {
                let named_fragments = db.ast_named_fragments(file_id);
                // TODO(@goto-bus-stop) Can we guarantee this unwrap()?
                let fragment_definition = named_fragments.get(&spread.fragment_name).unwrap();

                ApolloDiagnostic::new(
                    db,
                    spread.location(),
                    DiagnosticData::InvalidFragmentSpread {
                        name: Some(spread.fragment_name.to_string()),
                        type_name: against_type.to_string(),
                    },
                )
                .label(Label::new(
                    spread.location(),
                    format!("fragment `{}` cannot be applied", spread.fragment_name),
                ))
                .label(Label::new(
                    fragment_definition.location(),
                    format!("fragment declared with type condition `{type_condition}` here"),
                ))
                .label(Label::new(
                    against_type_definition.location(),
                    format!("type condition `{type_condition}` is not assignable to this type"),
                ))
            }
            ast::Selection::InlineFragment(inline) => ApolloDiagnostic::new(
                db,
                inline.location(),
                DiagnosticData::InvalidFragmentSpread {
                    name: None,
                    type_name: against_type.to_string(),
                },
            )
            .label(Label::new(
                inline.location(),
                format!("fragment applied with type condition `{type_condition}` here"),
            ))
            .label(Label::new(
                against_type_definition.location(),
                format!("type condition `{type_condition}` is not assignable to this type"),
            )),
        };

        diagnostics.push(diagnostic);
    }

    diagnostics
}

pub(crate) fn validate_inline_fragment(
    db: &dyn ValidationDatabase,
    file_id: FileId,
    against_type: Option<&ast::NamedType>,
    inline: Node<ast::InlineFragment>,
    context: OperationValidationConfig<'_>,
) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = Vec::new();

    diagnostics.extend(super::directive::validate_directives(
        db,
        inline.directives.iter(),
        ast::DirectiveLocation::InlineFragment,
        context.variables,
    ));

    let has_type_error = if context.has_schema {
        let type_cond_diagnostics = if let Some(t) = &inline.type_condition {
            validate_fragment_type_condition(db, t, inline.location())
        } else {
            Default::default()
        };
        let has_type_error = !type_cond_diagnostics.is_empty();
        diagnostics.extend(type_cond_diagnostics);
        has_type_error
    } else {
        false
    };

    // If there was an error with the type condition, it makes no sense to validate the selection,
    // as every field would be an error.
    if !has_type_error {
        if let (Some(against_type), Some(type_condition)) = (against_type, &inline.type_condition) {
            diagnostics.extend(validate_fragment_spread_type(
                db,
                file_id,
                against_type,
                type_condition,
                &ast::Selection::InlineFragment(inline.clone()),
            ));
        }
        diagnostics.extend(super::selection::validate_selection_set(
            db,
            file_id,
            inline.type_condition.as_ref().or(against_type),
            &inline.selection_set,
            context,
        ));
    }

    diagnostics
}

pub(crate) fn validate_fragment_spread(
    db: &dyn ValidationDatabase,
    file_id: FileId,
    against_type: Option<&ast::NamedType>,
    spread: Node<ast::FragmentSpread>,
    context: OperationValidationConfig<'_>,
) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = Vec::new();

    diagnostics.extend(super::directive::validate_directives(
        db,
        spread.directives.iter(),
        ast::DirectiveLocation::FragmentSpread,
        context.variables,
    ));

    let named_fragments = db.ast_named_fragments(file_id);
    match named_fragments.get(&spread.fragment_name) {
        Some(def) => {
            if let Some(against_type) = against_type {
                diagnostics.extend(validate_fragment_spread_type(
                    db,
                    file_id,
                    against_type,
                    &def.type_condition,
                    &ast::Selection::FragmentSpread(spread.clone()),
                ));
            }
            diagnostics.extend(validate_fragment_definition(
                db,
                file_id,
                def.clone(),
                context,
            ));
        }
        None => {
            diagnostics.push(
                ApolloDiagnostic::new(
                    db,
                    spread.location(),
                    DiagnosticData::UndefinedFragment {
                        name: spread.fragment_name.to_string(),
                    },
                )
                .labels(vec![Label::new(
                    spread.location(),
                    format!("fragment `{}` is not defined", spread.fragment_name),
                )]),
            );
        }
    }

    diagnostics
}

pub(crate) fn validate_fragment_definition(
    db: &dyn ValidationDatabase,
    file_id: FileId,
    fragment: Node<ast::FragmentDefinition>,
    context: OperationValidationConfig<'_>,
) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = Vec::new();
    let schema = db.schema();

    diagnostics.extend(super::directive::validate_directives(
        db,
        fragment.directives.iter(),
        ast::DirectiveLocation::FragmentDefinition,
        context.variables,
    ));

    let has_type_error = if context.has_schema {
        let type_cond_diagnostics =
            validate_fragment_type_condition(db, &fragment.type_condition, fragment.location());
        let has_type_error = !type_cond_diagnostics.is_empty();
        diagnostics.extend(type_cond_diagnostics);
        has_type_error
    } else {
        false
    };

    let fragment_cycles_diagnostics = validate_fragment_cycles(db, file_id, &fragment);
    let has_cycles = !fragment_cycles_diagnostics.is_empty();
    diagnostics.extend(fragment_cycles_diagnostics);

    if !has_type_error && !has_cycles {
        // If the type does not exist, do not attempt to validate the selections against it;
        // it has either already raised an error, or we are validating an executable without
        // a schema.
        let type_condition = schema
            .types
            .contains_key(&fragment.type_condition)
            .then_some(&fragment.type_condition);

        diagnostics.extend(super::selection::validate_selection_set(
            db,
            file_id,
            type_condition,
            &fragment.selection_set,
            context,
        ));
    }

    diagnostics
}

pub(crate) fn validate_fragment_cycles(
    db: &dyn ValidationDatabase,
    file_id: FileId,
    def: &Node<ast::FragmentDefinition>,
) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = Vec::new();

    // TODO pass in named fragments from outside this function, so it can be used on fully
    // synthetic trees.
    let named_fragments = db.ast_named_fragments(file_id);

    /// If a fragment spread is recursive, returns a vec containing the spread that refers back to
    /// the original fragment, and a trace of each fragment spread back to the original fragment.
    fn detect_fragment_cycles(
        named_fragments: &HashMap<ast::Name, Node<ast::FragmentDefinition>>,
        selection_set: &[ast::Selection],
        visited: &mut RecursionGuard<'_>,
    ) -> Result<(), CycleError<ast::FragmentSpread>> {
        for selection in selection_set {
            match selection {
                ast::Selection::FragmentSpread(spread) => {
                    if visited.contains(&spread.fragment_name) {
                        if visited.first() == Some(&spread.fragment_name) {
                            return Err(CycleError::Recursed(vec![spread.clone()]));
                        }
                        continue;
                    }

                    if let Some(fragment) = named_fragments.get(&spread.fragment_name) {
                        detect_fragment_cycles(
                            named_fragments,
                            &fragment.selection_set,
                            &mut visited.push(&fragment.name)?,
                        )
                        .map_err(|error| error.trace(spread))?;
                    }
                }
                ast::Selection::InlineFragment(inline) => {
                    detect_fragment_cycles(named_fragments, &inline.selection_set, visited)?;
                }
                ast::Selection::Field(field) => {
                    detect_fragment_cycles(named_fragments, &field.selection_set, visited)?;
                }
            }
        }

        Ok(())
    }

    let mut visited = RecursionStack::with_root(def.name.clone()).with_limit(100);

    match detect_fragment_cycles(&named_fragments, &def.selection_set, &mut visited.guard()) {
        Ok(_) => {}
        Err(CycleError::Recursed(trace)) => {
            let head_location = NodeLocation::recompose(def.location(), def.name.location());

            let mut diagnostic = ApolloDiagnostic::new(
                db,
                def.location(),
                DiagnosticData::RecursiveFragmentDefinition {
                    name: def.name.to_string(),
                },
            )
            .label(Label::new(head_location, "recursive fragment definition"));

            if let Some((cyclical_spread, path)) = trace.split_first() {
                let mut prev_name = &def.name;
                for spread in path.iter().rev() {
                    diagnostic = diagnostic.label(Label::new(
                        spread.location(),
                        format!(
                            "`{}` references `{}` here...",
                            prev_name, spread.fragment_name
                        ),
                    ));
                    prev_name = &spread.fragment_name;
                }

                diagnostic = diagnostic.label(Label::new(
                    cyclical_spread.location(),
                    format!(
                        "`{}` circularly references `{}` here",
                        prev_name, cyclical_spread.fragment_name
                    ),
                ));
            }

            diagnostics.push(diagnostic);
        }
        Err(CycleError::Limit(_)) => {
            let head_location = NodeLocation::recompose(def.location(), def.name.location());

            let diagnostic = ApolloDiagnostic::new(
                db,
                def.location(),
                DiagnosticData::DeeplyNestedType {
                    name: def.name.to_string(),
                },
            )
            .label(Label::new(
                head_location,
                "fragment references a very long chain of fragments",
            ));

            diagnostics.push(diagnostic);
        }
    }

    diagnostics
}

pub(crate) fn validate_fragment_type_condition(
    db: &dyn ValidationDatabase,
    type_cond: &ast::NamedType,
    fragment_location: Option<NodeLocation>,
) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = Vec::new();
    let schema = db.schema();

    let type_def = schema.types.get(type_cond);
    let is_composite = type_def
        .as_ref()
        // .map_or(false, |ty| ty.is_composite_definition());
        .map_or(false, |ty| {
            matches!(
                ty,
                schema::ExtendedType::Object(_)
                    | schema::ExtendedType::Interface(_)
                    | schema::ExtendedType::Union(_)
            )
        });

    if let Some(def) = type_def {
        if !is_composite {
            let diagnostic = ApolloDiagnostic::new(
                db,
                fragment_location,
                DiagnosticData::InvalidFragmentTarget {
                    ty: type_cond.to_string(),
                },
            )
            .label(Label::new(
                fragment_location,
                format!("fragment declares unsupported type condition `{type_cond}`"),
            ))
            .label(Label::new(
                def.location(),
                format!("`{type_cond}` is defined here"),
            ))
            .help("fragments cannot be defined on enums, scalars and input objects");
            diagnostics.push(diagnostic)
        }
    }

    diagnostics
}

pub(crate) fn validate_fragment_used(
    db: &dyn ValidationDatabase,
    fragment: Node<ast::FragmentDefinition>,
    file_id: FileId,
) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = Vec::new();

    let document = db.ast(file_id);
    let fragment_name = &fragment.name;

    let named_fragments = db.ast_named_fragments(file_id);
    let operations = document
        .definitions
        .iter()
        .filter_map(|definition| match definition {
            ast::Definition::OperationDefinition(operation) => Some(operation),
            _ => None,
        });

    let mut all_selections = operations
        .flat_map(|operation| &operation.selection_set)
        .chain(
            named_fragments
                .values()
                .flat_map(|fragment| &fragment.selection_set),
        );

    let is_used = all_selections.any(|sel| selection_uses_fragment(sel, fragment_name));

    // Fragments must be used within the schema
    //
    // Returns Unused Fragment error.
    if !is_used {
        diagnostics.push(
            ApolloDiagnostic::new(
                db,
                fragment.location(),
                DiagnosticData::UnusedFragment {
                    name: fragment_name.to_string(),
                },
            )
            .label(Label::new(
                fragment.location(),
                format!("`{fragment_name}` is defined here"),
            ))
            .help(format!(
                "fragment `{fragment_name}` must be used in an operation"
            )),
        )
    }
    diagnostics
}

fn selection_uses_fragment(sel: &ast::Selection, name: &str) -> bool {
    let sub_selections = match sel {
        ast::Selection::FragmentSpread(fragment) => return fragment.fragment_name == name,
        ast::Selection::Field(field) => &field.selection_set,
        ast::Selection::InlineFragment(inline) => &inline.selection_set,
    };

    sub_selections
        .iter()
        .any(|sel| selection_uses_fragment(sel, name))
}
