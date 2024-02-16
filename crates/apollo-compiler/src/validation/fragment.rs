use crate::ast;
use crate::ast::Name;
use crate::ast::NamedType;
use crate::executable;
use crate::schema;
use crate::schema::Implementers;
use crate::validation::diagnostics::{DiagnosticData, ValidationError};
use crate::validation::operation::OperationValidationConfig;
use crate::validation::{CycleError, NodeLocation, RecursionGuard, RecursionStack};
use crate::ExecutableDocument;
use crate::Node;
use std::borrow::Cow;
use std::collections::HashMap;
use std::collections::HashSet;

/// Given a type definition, find all the type names that can be used for fragment spreading.
///
/// Spec: https://spec.graphql.org/October2021/#GetPossibleTypes()
fn get_possible_types<'a>(
    type_definition: &schema::ExtendedType,
    implementers_map: &'a HashMap<Name, Implementers>,
) -> Cow<'a, HashSet<NamedType>> {
    match type_definition {
        // 1. If `type` is an object type, return a set containing `type`.
        schema::ExtendedType::Object(object) => {
            let mut set = HashSet::new();
            set.insert(object.name.clone());
            Cow::Owned(set)
        }
        // 2. If `type` is an interface type, return the set of object types implementing `type`.
        schema::ExtendedType::Interface(interface) => implementers_map
            .get(&interface.name)
            .map(|implementers| Cow::Borrowed(&implementers.objects))
            .unwrap_or_default(),
        // 3. If `type` is a union type, return the set of possible types of `type`.
        schema::ExtendedType::Union(union_) => Cow::Owned(
            union_
                .members
                .iter()
                .map(|component| component.name.clone())
                .collect(),
        ),
        _ => Default::default(),
    }
}

fn validate_fragment_spread_type(
    schema: &crate::Schema,
    document: &ExecutableDocument,
    against_type: &NamedType,
    type_condition: &NamedType,
    selection: &executable::Selection,
) -> Vec<ValidationError> {
    let mut diagnostics = Vec::new();

    // Another diagnostic will be raised if the type condition was wrong.
    // We reduce noise by silencing other issues with the fragment.
    let Some(type_condition_definition) = schema.types.get(type_condition) else {
        return diagnostics;
    };

    let Some(against_type_definition) = schema.types.get(against_type) else {
        // We cannot check anything if the parent type is unknown.
        return diagnostics;
    };

    let implementers_map = schema.implementers_map();
    let concrete_parent_types = get_possible_types(against_type_definition, &implementers_map);
    let concrete_condition_types = get_possible_types(type_condition_definition, &implementers_map);

    let mut applicable_types = concrete_parent_types.intersection(&concrete_condition_types);
    if applicable_types.next().is_none() {
        // Report specific errors for the different kinds of fragments.
        let diagnostic = match selection {
            executable::Selection::Field(_) => unreachable!(),
            executable::Selection::FragmentSpread(spread) => {
                // TODO(@goto-bus-stop) Can we guarantee this unwrap()?
                let fragment_definition = document.fragments.get(&spread.fragment_name).unwrap();

                ValidationError::new(
                    spread.location(),
                    DiagnosticData::InvalidFragmentSpread {
                        name: Some(spread.fragment_name.clone()),
                        type_name: against_type.clone(),
                        type_condition: type_condition.clone(),
                        fragment_location: fragment_definition.location(),
                        type_location: against_type_definition.location(),
                    },
                )
            }
            executable::Selection::InlineFragment(inline) => ValidationError::new(
                inline.location(),
                DiagnosticData::InvalidFragmentSpread {
                    name: None,
                    type_name: against_type.clone(),
                    type_condition: type_condition.clone(),
                    fragment_location: inline.location(),
                    type_location: against_type_definition.location(),
                },
            ),
        };

        diagnostics.push(diagnostic);
    }

    diagnostics
}

pub(crate) fn validate_inline_fragment(
    document: &ExecutableDocument,
    against_type: Option<(&crate::Schema, &ast::NamedType)>,
    inline: &Node<executable::InlineFragment>,
    context: OperationValidationConfig<'_>,
) -> Vec<ValidationError> {
    let mut diagnostics = Vec::new();

    diagnostics.extend(super::directive::validate_directives(
        context.schema,
        inline.directives.iter(),
        ast::DirectiveLocation::InlineFragment,
        context.variables,
    ));

    let has_type_error = if let Some(schema) = context.schema {
        let type_cond_diagnostics = if let Some(t) = &inline.type_condition {
            validate_fragment_type_condition(schema, None, t, inline.location())
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
        if let (Some((schema, against_type)), Some(type_condition)) =
            (against_type, &inline.type_condition)
        {
            diagnostics.extend(validate_fragment_spread_type(
                schema,
                document,
                against_type,
                type_condition,
                &executable::Selection::InlineFragment(inline.clone()),
            ));
        }
        diagnostics.extend(super::selection::validate_selection_set(
            document,
            if let (Some(schema), Some(ty)) = (&context.schema, &inline.type_condition) {
                Some((schema, ty))
            } else {
                against_type
            },
            &inline.selection_set,
            context,
        ));
    }

    diagnostics
}

pub(crate) fn validate_fragment_spread(
    document: &ExecutableDocument,
    against_type: Option<(&crate::Schema, &NamedType)>,
    spread: &Node<executable::FragmentSpread>,
    context: OperationValidationConfig<'_>,
) -> Vec<ValidationError> {
    let mut diagnostics = Vec::new();

    diagnostics.extend(super::directive::validate_directives(
        context.schema,
        spread.directives.iter(),
        ast::DirectiveLocation::FragmentSpread,
        context.variables,
    ));

    match document.fragments.get(&spread.fragment_name) {
        Some(def) => {
            if let Some((schema, against_type)) = against_type {
                diagnostics.extend(validate_fragment_spread_type(
                    schema,
                    document,
                    against_type,
                    def.type_condition(),
                    &executable::Selection::FragmentSpread(spread.clone()),
                ));
            }
            diagnostics.extend(validate_fragment_definition(document, def, context));
        }
        None => {
            diagnostics.push(ValidationError::new(
                spread.location(),
                DiagnosticData::UndefinedFragment {
                    name: spread.fragment_name.clone(),
                },
            ));
        }
    }

    diagnostics
}

pub(crate) fn validate_fragment_definition(
    document: &ExecutableDocument,
    fragment: &Node<executable::Fragment>,
    context: OperationValidationConfig<'_>,
) -> Vec<ValidationError> {
    let mut diagnostics = Vec::new();

    diagnostics.extend(super::directive::validate_directives(
        context.schema,
        fragment.directives.iter(),
        ast::DirectiveLocation::FragmentDefinition,
        context.variables,
    ));

    let has_type_error = if let Some(schema) = context.schema {
        let type_cond_diagnostics = validate_fragment_type_condition(
            schema,
            Some(fragment.name.clone()),
            fragment.type_condition(),
            fragment.location(),
        );
        let has_type_error = !type_cond_diagnostics.is_empty();
        diagnostics.extend(type_cond_diagnostics);
        has_type_error
    } else {
        false
    };

    let fragment_cycles_diagnostics = validate_fragment_cycles(document, fragment);
    let has_cycles = !fragment_cycles_diagnostics.is_empty();
    diagnostics.extend(fragment_cycles_diagnostics);

    if !has_type_error && !has_cycles {
        // If the type does not exist, do not attempt to validate the selections against it;
        // it has either already raised an error, or we are validating an executable without
        // a schema.
        let type_condition = context.schema.and_then(|schema| {
            schema
                .types
                .contains_key(fragment.type_condition())
                .then_some((schema, fragment.type_condition()))
        });

        diagnostics.extend(super::selection::validate_selection_set(
            document,
            type_condition,
            &fragment.selection_set,
            context,
        ));
    }

    diagnostics
}

pub(crate) fn validate_fragment_cycles(
    document: &ExecutableDocument,
    def: &Node<executable::Fragment>,
) -> Vec<ValidationError> {
    let mut diagnostics = Vec::new();

    /// If a fragment spread is recursive, returns a vec containing the spread that refers back to
    /// the original fragment, and a trace of each fragment spread back to the original fragment.
    fn detect_fragment_cycles(
        document: &ExecutableDocument,
        selection_set: &executable::SelectionSet,
        visited: &mut RecursionGuard<'_>,
    ) -> Result<(), CycleError<executable::FragmentSpread>> {
        for selection in &selection_set.selections {
            match selection {
                executable::Selection::FragmentSpread(spread) => {
                    if visited.contains(&spread.fragment_name) {
                        if visited.first() == Some(&spread.fragment_name) {
                            return Err(CycleError::Recursed(vec![spread.clone()]));
                        }
                        continue;
                    }

                    if let Some(fragment) = document.fragments.get(&spread.fragment_name) {
                        detect_fragment_cycles(
                            document,
                            &fragment.selection_set,
                            &mut visited.push(&fragment.name)?,
                        )
                        .map_err(|error| error.trace(spread))?;
                    }
                }
                executable::Selection::InlineFragment(inline) => {
                    detect_fragment_cycles(document, &inline.selection_set, visited)?;
                }
                executable::Selection::Field(field) => {
                    detect_fragment_cycles(document, &field.selection_set, visited)?;
                }
            }
        }

        Ok(())
    }

    let mut visited = RecursionStack::with_root(def.name.clone()).with_limit(100);

    match detect_fragment_cycles(document, &def.selection_set, &mut visited.guard()) {
        Ok(_) => {}
        Err(CycleError::Recursed(trace)) => {
            let head_location = NodeLocation::recompose(def.location(), def.name.location());

            diagnostics.push(ValidationError::new(
                def.location(),
                DiagnosticData::RecursiveFragmentDefinition {
                    head_location,
                    name: def.name.clone(),
                    trace,
                },
            ));
        }
        Err(CycleError::Limit(_)) => {
            let head_location = NodeLocation::recompose(def.location(), def.name.location());

            diagnostics.push(ValidationError::new(
                head_location,
                DiagnosticData::DeeplyNestedType {
                    name: def.name.clone(),
                    describe_type: "fragment",
                },
            ));
        }
    }

    diagnostics
}

pub(crate) fn validate_fragment_type_condition(
    schema: &crate::Schema,
    fragment_name: Option<Name>,
    type_cond: &NamedType,
    fragment_location: Option<NodeLocation>,
) -> Vec<ValidationError> {
    let mut diagnostics = Vec::new();

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

    if !is_composite {
        diagnostics.push(ValidationError::new(
            fragment_location,
            DiagnosticData::InvalidFragmentTarget {
                name: fragment_name,
                ty: type_cond.clone(),
            },
        ));
    }

    diagnostics
}

pub(crate) fn validate_fragment_used(
    document: &ExecutableDocument,
    fragment: &Node<executable::Fragment>,
) -> Vec<ValidationError> {
    let mut diagnostics = Vec::new();

    let fragment_name = &fragment.name;

    let mut all_selections = document
        .all_operations()
        .map(|operation| &operation.selection_set)
        .chain(
            document
                .fragments
                .values()
                .map(|fragment| &fragment.selection_set),
        )
        .flat_map(|set| &set.selections);

    let is_used = all_selections.any(|sel| selection_uses_fragment(sel, fragment_name));

    // Fragments must be used within the schema
    //
    // Returns Unused Fragment error.
    if !is_used {
        diagnostics.push(ValidationError::new(
            fragment.location(),
            DiagnosticData::UnusedFragment {
                name: fragment_name.clone(),
            },
        ))
    }
    diagnostics
}

fn selection_uses_fragment(sel: &executable::Selection, name: &str) -> bool {
    let sub_selections = match sel {
        executable::Selection::FragmentSpread(fragment) => return fragment.fragment_name == name,
        executable::Selection::Field(field) => &field.selection_set,
        executable::Selection::InlineFragment(inline) => &inline.selection_set,
    };

    sub_selections
        .selections
        .iter()
        .any(|sel| selection_uses_fragment(sel, name))
}
