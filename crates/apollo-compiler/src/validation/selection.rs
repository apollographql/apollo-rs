use std::sync::Arc;

use crate::{hir, validation::ValidationDatabase, ApolloDiagnostic};

pub fn validate_selection(
    db: &dyn ValidationDatabase,
    selection: Arc<Vec<hir::Selection>>,
) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = Vec::new();

    for sel in selection.iter() {
        match sel {
            hir::Selection::Field(field) => {
                diagnostics.extend(db.validate_field(field.clone()));
            }

            // TODO handle fragment spreads on invalid parent types
            hir::Selection::FragmentSpread(frag) => diagnostics.extend(db.validate_directives(
                frag.directives().to_vec(),
                hir::DirectiveLocation::FragmentSpread,
            )),
            hir::Selection::InlineFragment(inline) => diagnostics.extend(db.validate_directives(
                inline.directives().to_vec(),
                hir::DirectiveLocation::InlineFragment,
            )),
        }
    }

    diagnostics
}

pub fn validate_selection_set(
    db: &dyn ValidationDatabase,
    selection_set: hir::SelectionSet,
) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = Vec::new();

    diagnostics.extend(db.validate_selection(selection_set.selection));

    diagnostics
}
