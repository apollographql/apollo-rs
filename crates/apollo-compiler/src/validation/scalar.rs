use crate::{
    diagnostics::{BuiltInScalarDefinition, Diagnostic2, DiagnosticData, Label},
    ApolloDiagnostic, ValidationDatabase,
};

const BUILT_IN_SCALARS: [&str; 5] = ["Int", "Float", "Boolean", "String", "ID"];

pub fn check(db: &dyn ValidationDatabase) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = Vec::new();

    for (name, scalar) in db.scalars().iter() {
        if let Some(loc) = scalar.loc() {
            let offset = loc.offset();
            let len = loc.node_len();

            // All built-in scalars must be omitted for brevity.
            if BUILT_IN_SCALARS.contains(&&**name) && !scalar.is_built_in() {
                diagnostics.push(ApolloDiagnostic::BuiltInScalarDefinition(
                    BuiltInScalarDefinition {
                        scalar: (offset, len).into(),
                        src: db.source_code(loc.file_id()),
                    },
                ));
            } else if !scalar.is_built_in() {
                // Custom scalars must provide a scalar specification URL via the
                // @specifiedBy directive
                if !scalar
                    .directives()
                    .iter()
                    .any(|directive| directive.name() == "specifiedBy")
                {
                    diagnostics.push(ApolloDiagnostic::Diagnostic2(
                        Diagnostic2::new(db, loc.into(), DiagnosticData::ScalarSpecificationURL)
                            .label(Label::new(
                            loc,
                            "consider adding a @specifiedBy directive to this scalar definition",
                        )),
                    ))
                }
            }
        }
    }

    diagnostics
}
