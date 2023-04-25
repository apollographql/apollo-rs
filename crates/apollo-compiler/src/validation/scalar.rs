use std::sync::Arc;

use crate::{
    diagnostics::{ApolloDiagnostic, DiagnosticData, Label},
    hir::{self, DirectiveLocation},
    ValidationDatabase,
};

pub fn validate_scalar_definitions(db: &dyn ValidationDatabase) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = Vec::new();

    let defs = &db.type_system_definitions().scalars;
    for def in defs.values() {
        diagnostics.extend(db.validate_scalar_definition(def.clone()));
    }

    diagnostics
}

pub fn validate_scalar_definition(
    db: &dyn ValidationDatabase,
    scalar_def: Arc<hir::ScalarTypeDefinition>,
) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = Vec::new();

    // All built-in scalars must be omitted for brevity.
    if !scalar_def.is_built_in() {
        // Custom scalars must provide a scalar specification URL via the
        // @specifiedBy directive
        if !scalar_def
            .directives()
            .any(|directive| directive.name() == "specifiedBy")
        {
            diagnostics.push(
                ApolloDiagnostic::new(
                    db,
                    scalar_def.loc.into(),
                    DiagnosticData::ScalarSpecificationURL,
                )
                .label(Label::new(
                    scalar_def.loc,
                    "consider adding a @specifiedBy directive to this scalar definition",
                )),
            )
        }

        diagnostics.extend(db.validate_directives(
            scalar_def.directives().cloned().collect(),
            DirectiveLocation::Scalar,
            // scalars don't use variables
            Arc::new(Vec::new()),
        ));
    }

    diagnostics
}
