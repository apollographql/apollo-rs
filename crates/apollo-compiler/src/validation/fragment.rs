use crate::{
    diagnostics::{ApolloDiagnostic, DiagnosticData, Label},
    hir,
    validation::RecursionStack,
    FileId, ValidationDatabase,
};
use std::{collections::HashSet, sync::Arc};

/// Given a type definition, find all the types that can be used for fragment spreading.
///
/// Spec: https://spec.graphql.org/October2021/#GetPossibleTypes()
pub fn get_possible_types(
    db: &dyn ValidationDatabase,
    ty: hir::TypeDefinition,
) -> Vec<hir::TypeDefinition> {
    fn get_possible_types_impl(
        db: &dyn ValidationDatabase,
        ty: hir::TypeDefinition,
        seen: &mut RecursionStack<'_>,
        output: &mut Vec<hir::TypeDefinition>,
    ) {
        match &ty {
            // 1. If `type` is an object type, return a set containing `type`.
            hir::TypeDefinition::ObjectTypeDefinition(_) => output.push(ty),
            // 2. If `type` is an interface type, return the set of types implementing `type`.
            hir::TypeDefinition::InterfaceTypeDefinition(intf) => {
                // Prevent stack overflow if interface implements itself
                if seen.contains(intf.name()) {
                    return;
                }

                let subtype_map = db.subtype_map();
                if let Some(names) = subtype_map.get(intf.name()) {
                    let mut seen = seen.push(intf.name().to_string());
                    names
                        .iter()
                        .filter_map(|name| db.find_type_definition_by_name(name.to_string()))
                        .for_each(|ty| get_possible_types_impl(db, ty, &mut seen, output))
                }
                output.push(ty);
            }
            // 3. If `type` is a union type, return the set of possible types of `type`.
            hir::TypeDefinition::UnionTypeDefinition(union_) => {
                // Prevent stack overflow if union is a member of itself
                if seen.contains(union_.name()) {
                    return;
                }

                let subtype_map = db.subtype_map();
                if let Some(names) = subtype_map.get(union_.name()) {
                    let mut seen = seen.push(union_.name().to_string());
                    names
                        .iter()
                        .filter_map(|name| db.find_type_definition_by_name(name.to_string()))
                        .for_each(|ty| get_possible_types_impl(db, ty, &mut seen, output))
                }

                output.push(ty);
            }
            _ => (),
        }
    }

    let mut output = vec![];
    get_possible_types_impl(db, ty, &mut RecursionStack(&mut vec![]), &mut output);
    output
}

pub fn validate_fragment_selection(
    db: &dyn ValidationDatabase,
    spread: hir::FragmentSelection,
) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = Vec::new();

    let Some(cond) = spread.type_condition(db.upcast()) else {
        // Returns None on invalid documents only
        return diagnostics;
    };
    let Some(parent_type) = spread.parent_type(db.upcast()) else {
        // We cannot check anything if the parent type is unknown.
        return diagnostics;
    };
    let Some(cond_type) = db.find_type_definition_by_name(cond.clone()) else {
        // We cannot check anything if the type condition refers to an unknown type.
        return diagnostics;
    };

    let concrete_parent_types = db
        .get_possible_types(parent_type.clone())
        .into_iter()
        .map(|ty| ty.name().to_string())
        .collect::<HashSet<_>>();
    let concrete_condition_types = db
        .get_possible_types(cond_type)
        .into_iter()
        .map(|ty| ty.name().to_string())
        .collect::<HashSet<_>>();

    let mut applicable_types = concrete_parent_types.intersection(&concrete_condition_types);
    if applicable_types.next().is_none() {
        // Report specific errors for the different kinds of fragments.
        let diagnostic = match spread {
            hir::FragmentSelection::FragmentSpread(spread) => {
                // This unwrap is safe because the fragment definition must exist for `cond_type` to be `Some()`.
                let fragment_definition = spread.fragment(db.upcast()).unwrap();

                ApolloDiagnostic::new(
                    db,
                    spread.loc().into(),
                    DiagnosticData::InvalidFragmentSpread {
                        name: Some(spread.name().to_string()),
                        type_name: parent_type.name().to_string(),
                    },
                )
                .label(Label::new(
                    spread.loc(),
                    format!("fragment `{}` cannot be applied", spread.name()),
                ))
                .label(Label::new(
                    fragment_definition.loc(),
                    format!("fragment declared with type condition `{cond}` here"),
                ))
                .label(Label::new(
                    parent_type.loc(),
                    format!("type condition `{cond}` is not assignable to this type"),
                ))
            }
            hir::FragmentSelection::InlineFragment(inline) => ApolloDiagnostic::new(
                db,
                inline.loc().into(),
                DiagnosticData::InvalidFragmentSpread {
                    name: None,
                    type_name: parent_type.name().to_string(),
                },
            )
            .label(Label::new(
                inline.loc(),
                format!("fragment applied with type condition `{cond}` here"),
            ))
            .label(Label::new(
                parent_type.loc(),
                format!("type condition `{cond}` is not assignable to this type"),
            )),
        };

        diagnostics.push(diagnostic);
    }

    diagnostics
}

pub fn validate_inline_fragment(
    db: &dyn ValidationDatabase,
    inline: Arc<hir::InlineFragment>,
    var_defs: Arc<Vec<hir::VariableDefinition>>,
) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = Vec::new();

    diagnostics.extend(db.validate_directives(
        inline.directives().to_vec(),
        hir::DirectiveLocation::InlineFragment,
        var_defs.clone(),
    ));

    let type_cond_diagnostics = db.validate_fragment_type_condition(
        inline.type_condition().map(|t| t.to_string()),
        inline.loc(),
    );
    let has_type_error = !type_cond_diagnostics.is_empty();
    diagnostics.extend(type_cond_diagnostics);

    diagnostics.extend(db.validate_selection_set(inline.selection_set.clone(), var_defs));
    // If there was an error with the type condition, it makes no sense to validate the selection,
    // as every field would be an error.
    if !has_type_error {
        diagnostics.extend(db.validate_selection_set(inline.selection_set.clone(), var_defs));
        diagnostics
            .extend(db.validate_fragment_selection(hir::FragmentSelection::InlineFragment(inline)));
    }

    diagnostics
}

pub fn validate_fragment_spread(
    db: &dyn ValidationDatabase,
    spread: Arc<hir::FragmentSpread>,
    var_defs: Arc<Vec<hir::VariableDefinition>>,
) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = Vec::new();

    diagnostics.extend(db.validate_directives(
        spread.directives().to_vec(),
        hir::DirectiveLocation::FragmentSpread,
        var_defs.clone(),
    ));

    match spread.fragment(db.upcast()) {
        Some(def) => {
            diagnostics.extend(
                db.validate_fragment_selection(hir::FragmentSelection::FragmentSpread(spread)),
            );
            diagnostics.extend(db.validate_fragment_definition(def, var_defs));
        }
        None => {
            diagnostics.push(
                ApolloDiagnostic::new(
                    db,
                    spread.loc().into(),
                    DiagnosticData::UndefinedFragment {
                        name: spread.name().to_string(),
                    },
                )
                .labels(vec![Label::new(
                    spread.loc(),
                    format!("fragment `{}` is not defined", spread.name()),
                )]),
            );
        }
    }

    diagnostics
}

pub fn validate_fragment_definition(
    db: &dyn ValidationDatabase,
    def: Arc<hir::FragmentDefinition>,
    var_defs: Arc<Vec<hir::VariableDefinition>>,
) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = Vec::new();
    diagnostics.extend(db.validate_directives(
        def.directives().to_vec(),
        hir::DirectiveLocation::FragmentDefinition,
        var_defs.clone(),
    ));

    let type_cond_diagnostics =
        db.validate_fragment_type_condition(Some(def.type_condition().to_string()), def.loc());
    let has_type_error = !type_cond_diagnostics.is_empty();
    diagnostics.extend(type_cond_diagnostics);

    let fragment_cycles_diagnostics = db.validate_fragment_cycles(def.clone());
    let has_cycles = !fragment_cycles_diagnostics.is_empty();
    diagnostics.extend(fragment_cycles_diagnostics);

    if !has_type_error && !has_cycles {
        diagnostics.extend(db.validate_selection_set(def.selection_set().clone(), var_defs));
    }

    diagnostics.extend(db.validate_fragment_cycles(def));

    diagnostics
}

pub fn validate_fragment_cycles(
    db: &dyn ValidationDatabase,
    def: Arc<hir::FragmentDefinition>,
) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = Vec::new();

    fn detect_fragment_cycles(
        db: &dyn ValidationDatabase,
        selection_set: &hir::SelectionSet,
        visited: &mut RecursionStack<'_>,
    ) -> Result<(), hir::Selection> {
        for selection in selection_set.selection() {
            match selection {
                hir::Selection::FragmentSpread(spread) => {
                    if visited.contains(spread.name()) {
                        if visited.first() == Some(spread.name()) {
                            return Err(selection.clone());
                        }
                        continue;
                    }

                    if let Some(fragment) =
                        db.find_fragment_by_name(spread.loc().file_id(), spread.name().to_string())
                    {
                        detect_fragment_cycles(
                            db,
                            fragment.selection_set(),
                            &mut visited.push(fragment.name().to_string()),
                        )?;
                    }
                }
                hir::Selection::InlineFragment(inline) => {
                    detect_fragment_cycles(db, inline.selection_set(), visited)?;
                }
                _ => {}
            }
        }

        Ok(())
    }

    // Split RecursionStack initialisation for lifetime reasons
    let mut visited = vec![];
    let mut visited = RecursionStack(&mut visited);
    let mut visited = visited.push(def.name().to_string());

    if let Err(cycle) = detect_fragment_cycles(db, def.selection_set(), &mut visited) {
        diagnostics.push(
            ApolloDiagnostic::new(
                db,
                def.loc().into(),
                DiagnosticData::RecursiveFragmentDefinition {
                    name: def.name().to_string(),
                },
            )
            .label(Label::new(def.head_loc(), "recursive fragment definition"))
            .label(Label::new(cycle.loc(), "refers to itself here")),
        );
    }

    diagnostics
}

pub fn validate_fragment_type_condition(
    db: &dyn ValidationDatabase,
    type_cond: Option<String>,
    loc: hir::HirNodeLocation,
) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = Vec::new();
    let schema_types = db.types_definitions_by_name();

    match type_cond {
        Some(type_cond) => {
            let type_def = db.find_type_definition_by_name(type_cond.clone());
            let is_composite = type_def
                .as_ref()
                .map_or(false, |ty| ty.is_composite_definition());

            if !schema_types.contains_key(&type_cond) {
                diagnostics.push(
                    ApolloDiagnostic::new(
                        db,
                        loc.into(),
                        DiagnosticData::InvalidFragment {
                            ty: type_cond.clone().into(),
                        },
                    )
                    .label(Label::new(
                        loc,
                        format!("`{type_cond}` is defined here but not declared in the schema"),
                    ))
                    .help("fragments must be specified on types that exist in the schema")
                    .help(format!("consider defining `{type_cond}` in the schema")),
                );
            } else if !is_composite {
                let mut diagnostic = ApolloDiagnostic::new(
                    db,
                    loc.into(),
                    DiagnosticData::InvalidFragmentTarget {
                        ty: type_cond.clone(),
                    },
                )
                .label(Label::new(
                    loc,
                    format!(
                        "fragment declares unsupported type condition `{}`",
                        type_cond
                    ),
                ))
                .help("fragments cannot be defined on enums, scalars and input object");
                if let Some(def) = type_def {
                    diagnostic = diagnostic.label(Label::new(
                        def.loc(),
                        format!("`{type_cond}` is defined here"),
                    ))
                }
                diagnostics.push(diagnostic)
            }
        }
        None => {
            diagnostics.push(
                ApolloDiagnostic::new(db, loc.into(), DiagnosticData::InvalidFragment { ty: None })
                    .label(Label::new(
                        loc,
                        "fragment target is defined here but not declared in the schema"
                            .to_string(),
                    )),
            );
        }
    }

    diagnostics
}

pub fn validate_fragment_used(
    db: &dyn ValidationDatabase,
    def: Arc<hir::FragmentDefinition>,
    file_id: FileId,
) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = Vec::new();

    let operations = db.operations(file_id);
    let fragments = db.fragments(file_id);
    let name = def.name();

    // Fragments must be used within the schema
    //
    // Returns Unused Fragment error.
    if !operations.iter().any(|op| {
        op.selection_set()
            .selection()
            .iter()
            .any(|sel| is_fragment_used(sel.clone(), name))
    }) & !fragments.values().any(|op| {
        op.selection_set()
            .selection()
            .iter()
            .any(|sel| is_fragment_used(sel.clone(), name))
    }) {
        diagnostics.push(
            ApolloDiagnostic::new(
                db,
                def.loc().into(),
                DiagnosticData::UnusedFragment { name: name.into() },
            )
            .label(Label::new(def.loc(), format!("`{name}` is defined here")))
            .help(format!("fragment `{name}` must be used in an operation")),
        )
    }
    diagnostics
}

fn is_fragment_used(sel: hir::Selection, name: &str) -> bool {
    match sel {
        hir::Selection::Field(field) => {
            let sel = field.selection_set().selection();
            sel.iter().any(|sel| is_fragment_used(sel.clone(), name))
        }
        hir::Selection::FragmentSpread(fragment) => {
            if fragment.name() == name {
                return true;
            }
            false
        }
        hir::Selection::InlineFragment(inline) => {
            let sel = inline.selection_set().selection();
            sel.iter().any(|sel| is_fragment_used(sel.clone(), name))
        }
    }
}
