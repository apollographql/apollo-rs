use crate::{hir, validation::ValidationDatabase, ApolloDiagnostic};

pub fn validate(
    db: &dyn ValidationDatabase,
    selection: Vec<hir::Selection>,
) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = Vec::new();

    for sel in selection {
        match sel {
            hir::Selection::Field(field) => {
                if !field.selection_set().selection().is_empty() {
                    diagnostics.extend(validate(db, (*field.selection_set().selection).clone()))
                }
                diagnostics.extend(db.validate_field(field.as_ref().clone()));
            }
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
