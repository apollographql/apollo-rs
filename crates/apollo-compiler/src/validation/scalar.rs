use crate::{
    diagnostics::{BuiltInScalarDefinition, ScalarSpecificationURL},
    hir::{self, DirectiveLocation},
    ApolloDiagnostic, ValidationDatabase,
};

const BUILT_IN_SCALARS: [&str; 5] = ["Int", "Float", "Boolean", "String", "ID"];

pub fn validate_scalar_definitions(db: &dyn ValidationDatabase) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = Vec::new();

    let defs = &db.type_system_definitions().scalars;
    for def in defs.values() {
        diagnostics.extend(db.validate_scalar_definition(def.as_ref().clone()));
    }

    diagnostics
}

pub fn validate_scalar_definition(
    db: &dyn ValidationDatabase,
    scalar_def: hir::ScalarTypeDefinition,
) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = Vec::new();
    let name = scalar_def.name();

    if let Some(loc) = scalar_def.loc() {
        let offset = loc.offset();
        let len = loc.node_len();

        // All built-in scalars must be omitted for brevity.
        if BUILT_IN_SCALARS.contains(&name) && !scalar_def.is_built_in() {
            diagnostics.push(ApolloDiagnostic::BuiltInScalarDefinition(
                BuiltInScalarDefinition {
                    scalar: (offset, len).into(),
                    src: db.source_code(loc.file_id()),
                },
            ));
        } else if !scalar_def.is_built_in() {
            // Custom scalars must provide a scalar specification URL via the
            // @specifiedBy directive
            if !scalar_def
                .directives()
                .iter()
                .any(|directive| directive.name() == "specifiedBy")
            {
                diagnostics.push(ApolloDiagnostic::ScalarSpecificationURL(
                    ScalarSpecificationURL {
                        scalar: (offset, len).into(),
                        src: db.source_code(loc.file_id()),
                    },
                ))
            }

            diagnostics.extend(
                db.validate_directives(scalar_def.directives().to_vec(), DirectiveLocation::Scalar),
            );
        }
    }

    diagnostics
}
