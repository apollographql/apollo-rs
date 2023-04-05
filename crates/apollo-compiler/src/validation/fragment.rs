use crate::{
    diagnostics::{ApolloDiagnostic, DiagnosticData, Label},
    hir,
    validation::RecursionStack,
    FileId, ValidationDatabase,
};
use std::sync::Arc;

pub fn validate_fragment_spread(
    db: &dyn ValidationDatabase,
    spread: Arc<hir::FragmentSpread>,
) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = Vec::new();

    diagnostics.extend(db.validate_directives(
        spread.directives().to_vec(),
        hir::DirectiveLocation::FragmentSpread,
    ));

    if spread.fragment(db.upcast()).is_none() {
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

    diagnostics
}

pub fn validate_fragment_definitions(
    db: &dyn ValidationDatabase,
    file_id: FileId,
) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = Vec::new();
    for def in db.fragments(file_id).values() {
        diagnostics.extend(db.validate_directives(
            def.directives().to_vec(),
            hir::DirectiveLocation::FragmentDefinition,
        ));
        diagnostics.extend(
            db.validate_fragment_type_condition(Some(def.type_condition().to_string()), def.loc()),
        );
        diagnostics.extend(db.validate_selection_set(def.selection_set().clone()));
        diagnostics.extend(db.validate_fragment_used(Arc::clone(def), file_id));

        diagnostics.extend(db.validate_fragment_cycles(Arc::clone(def)));
    }

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
            .label(Label::new(
                def.head_loc(),
                "recursive fragment definition",
            ))
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
                .clone()
                .map_or(false, |ty| ty.is_composite_definition());

            if !is_composite {
                let mut diagnostic = ApolloDiagnostic::new(
                    db,
                    loc.into(),
                    DiagnosticData::InvalidFragmentTarget {
                        ty: type_cond.clone(),
                    },
                )
                .help("fragments cannot be defined on enums, scalars and input object");
                if let Some(def) = type_def {
                    diagnostic = diagnostic.label(Label::new(
                        def.loc(),
                        format!("`{type_cond}` is defined here"),
                    ))
                }
                diagnostics.push(diagnostic)
            }

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
                        format!(
                            "`{}` is defined here but not declared in the schema",
                            &type_cond
                        ),
                    ))
                    .help(
                        "fragments must be specified on types that exist in the schema".to_string(),
                    )
                    .help(format!("consider defining `{}` in the schema", &type_cond)),
                );
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
