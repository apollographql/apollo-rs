use crate::ast;
use crate::ast::NamedType;
use crate::executable;
use crate::schema;
use crate::schema::Implementers;
use crate::validation::diagnostics::DiagnosticData;
use crate::validation::CycleError;
use crate::validation::DiagnosticList;
use crate::validation::NodeLocation;
use crate::validation::OperationValidationContext;
use crate::validation::RecursionGuard;
use crate::validation::RecursionStack;
use crate::ExecutableDocument;
use crate::Name;
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
    diagnostics: &mut DiagnosticList,
    schema: &crate::Schema,
    document: &ExecutableDocument,
    against_type: &NamedType,
    type_condition: &NamedType,
    selection: &executable::Selection,
    context: OperationValidationContext<'_>,
) {
    // Another diagnostic will be raised if the type condition was wrong.
    // We reduce noise by silencing other issues with the fragment.
    let Some(type_condition_definition) = schema.types.get(type_condition) else {
        return;
    };

    let Some(against_type_definition) = schema.types.get(against_type) else {
        // We cannot check anything if the parent type is unknown.
        return;
    };

    let implementers_map = context.implementers_map();
    let concrete_parent_types = get_possible_types(against_type_definition, implementers_map);
    let concrete_condition_types = get_possible_types(type_condition_definition, implementers_map);

    let mut applicable_types = concrete_parent_types.intersection(&concrete_condition_types);
    if applicable_types.next().is_none() {
        // Report specific errors for the different kinds of fragments.
        match selection {
            executable::Selection::Field(_) => unreachable!(),
            executable::Selection::FragmentSpread(spread) => {
                // TODO(@goto-bus-stop) Can we guarantee this unwrap()?
                let fragment_definition = document.fragments.get(&spread.fragment_name).unwrap();

                diagnostics.push(
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
            executable::Selection::InlineFragment(inline) => diagnostics.push(
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
    }
}

pub(crate) fn validate_inline_fragment(
    diagnostics: &mut DiagnosticList,
    document: &ExecutableDocument,
    against_type: Option<(&crate::Schema, &ast::NamedType)>,
    inline: &Node<executable::InlineFragment>,
    context: OperationValidationContext<'_>,
) {
    super::directive::validate_directives(
        diagnostics,
        context.schema(),
        inline.directives.iter(),
        ast::DirectiveLocation::InlineFragment,
        context.variables,
    );

    let previous = diagnostics.len();
    if let Some(schema) = context.schema() {
        if let Some(t) = &inline.type_condition {
            validate_fragment_type_condition(diagnostics, schema, None, t, inline.location())
        }
    }
    let has_type_error = diagnostics.len() > previous;

    // If there was an error with the type condition, it makes no sense to validate the selection,
    // as every field would be an error.
    if !has_type_error {
        if let (Some((schema, against_type)), Some(type_condition)) =
            (against_type, &inline.type_condition)
        {
            validate_fragment_spread_type(
                diagnostics,
                schema,
                document,
                against_type,
                type_condition,
                &executable::Selection::InlineFragment(inline.clone()),
                context,
            );
        }
        super::selection::validate_selection_set(
            diagnostics,
            document,
            if let (Some(schema), Some(ty)) = (&context.schema(), &inline.type_condition) {
                Some((schema, ty))
            } else {
                against_type
            },
            &inline.selection_set,
            context,
        );
    }
}

pub(crate) fn validate_fragment_spread(
    diagnostics: &mut DiagnosticList,
    document: &ExecutableDocument,
    against_type: Option<(&crate::Schema, &NamedType)>,
    spread: &Node<executable::FragmentSpread>,
    context: OperationValidationContext<'_>,
) {
    super::directive::validate_directives(
        diagnostics,
        context.schema(),
        spread.directives.iter(),
        ast::DirectiveLocation::FragmentSpread,
        context.variables,
    );

    match document.fragments.get(&spread.fragment_name) {
        Some(def) => {
            if let Some((schema, against_type)) = against_type {
                validate_fragment_spread_type(
                    diagnostics,
                    schema,
                    document,
                    against_type,
                    def.type_condition(),
                    &executable::Selection::FragmentSpread(spread.clone()),
                    context,
                );
            }
            validate_fragment_definition(diagnostics, document, def, context);
        }
        None => {
            diagnostics.push(
                spread.location(),
                DiagnosticData::UndefinedFragment {
                    name: spread.fragment_name.clone(),
                },
            );
        }
    }
}

pub(crate) fn validate_fragment_definition(
    diagnostics: &mut DiagnosticList,
    document: &ExecutableDocument,
    fragment: &Node<executable::Fragment>,
    context: OperationValidationContext<'_>,
) {
    super::directive::validate_directives(
        diagnostics,
        context.schema(),
        fragment.directives.iter(),
        ast::DirectiveLocation::FragmentDefinition,
        context.variables,
    );

    let previous = diagnostics.len();
    if let Some(schema) = context.schema() {
        validate_fragment_type_condition(
            diagnostics,
            schema,
            Some(fragment.name.clone()),
            fragment.type_condition(),
            fragment.location(),
        );
    }
    let has_type_error = diagnostics.len() > previous;

    let previous = diagnostics.len();
    validate_fragment_cycles(diagnostics, document, fragment);
    let has_cycles = diagnostics.len() > previous;

    if !has_type_error && !has_cycles {
        // If the type does not exist, do not attempt to validate the selections against it;
        // it has either already raised an error, or we are validating an executable without
        // a schema.
        let type_condition = context.schema().and_then(|schema| {
            schema
                .types
                .contains_key(fragment.type_condition())
                .then_some((schema, fragment.type_condition()))
        });

        super::selection::validate_selection_set(
            diagnostics,
            document,
            type_condition,
            &fragment.selection_set,
            context,
        );
    }
}

pub(crate) fn validate_fragment_cycles(
    diagnostics: &mut DiagnosticList,
    document: &ExecutableDocument,
    def: &Node<executable::Fragment>,
) {
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

            diagnostics.push(
                def.location(),
                DiagnosticData::RecursiveFragmentDefinition {
                    head_location,
                    name: def.name.clone(),
                    trace,
                },
            );
        }
        Err(CycleError::Limit(_)) => {
            let head_location = NodeLocation::recompose(def.location(), def.name.location());

            diagnostics.push(
                head_location,
                DiagnosticData::DeeplyNestedType {
                    name: def.name.clone(),
                    describe_type: "fragment",
                },
            );
        }
    };
}

pub(crate) fn validate_fragment_type_condition(
    diagnostics: &mut DiagnosticList,
    schema: &crate::Schema,
    fragment_name: Option<Name>,
    type_cond: &NamedType,
    fragment_location: Option<NodeLocation>,
) {
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
        diagnostics.push(
            fragment_location,
            DiagnosticData::InvalidFragmentTarget {
                name: fragment_name,
                ty: type_cond.clone(),
            },
        );
    }
}

pub(crate) fn validate_fragment_used(
    diagnostics: &mut DiagnosticList,
    document: &ExecutableDocument,
    fragment: &Node<executable::Fragment>,
) {
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
        diagnostics.push(
            fragment.location(),
            DiagnosticData::UnusedFragment {
                name: fragment_name.clone(),
            },
        )
    }
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
