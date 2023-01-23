use crate::{
    hir,
    validation::{selection, ValidationDatabase},
    ApolloDiagnostic,
};

pub fn validate(
    db: &dyn ValidationDatabase,
    selection_set: hir::SelectionSet,
) -> Vec<ApolloDiagnostic> {
    selection::validate(db, (*selection_set.selection).clone())
}
