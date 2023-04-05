use crate::{
    diagnostics::{ApolloDiagnostic, DiagnosticData, Label},
    hir, FileId, ValidationDatabase,
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
        diagnostics.extend(db.validate_fragment_used(def.as_ref().clone(), file_id));
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
    def: hir::FragmentDefinition,
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
    }) & !fragments.values().into_iter().any(|op| {
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
