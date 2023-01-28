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

        let fragment_type_def = db.find_type_definition_by_name(def.type_condition().to_string());
        if let Some(fragment_type_def) = fragment_type_def {
            diagnostics
                .extend(db.validate_selection_set(def.selection_set().clone(), fragment_type_def));
        } else {
            // TODO what should we do if fragment_type_def is None although fragment_type is Some? Is that a case we are expecting?
        }
    }

    diagnostics
}
