use crate::{hir::DirectiveLocation, ApolloDiagnostic, FileId, ValidationDatabase};

pub fn validate_fragment_definitions(
    db: &dyn ValidationDatabase,
    file_id: FileId,
) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = Vec::new();
    for def in db.fragments(file_id).values() {
        diagnostics.extend(db.validate_directives(
            def.directives().to_vec(),
            DirectiveLocation::FragmentDefinition,
        ));
        diagnostics.extend(db.validate_selection_set(def.selection_set().clone()));
    }

    diagnostics
}
