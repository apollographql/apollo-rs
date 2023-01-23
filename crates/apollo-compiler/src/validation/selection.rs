use crate::{
    hir,
    validation::{directive, field, ValidationDatabase},
    ApolloDiagnostic,
};

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
                diagnostics.extend(field::validate(db, field));
            }
            hir::Selection::FragmentSpread(frag) => diagnostics.extend(directive::validate_usage(
                db,
                frag.directives().to_vec(),
                hir::DirectiveLocation::FragmentSpread,
            )),
            hir::Selection::InlineFragment(inline) => {
                diagnostics.extend(directive::validate_usage(
                    db,
                    inline.directives().to_vec(),
                    hir::DirectiveLocation::InlineFragment,
                ))
            }
        }
    }

    diagnostics
}
