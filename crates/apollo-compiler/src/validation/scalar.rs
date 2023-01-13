use crate::{
    diagnostics::{ApolloDiagnostic, DiagnosticData, Label},
    ValidationDatabase,
};

const BUILT_IN_SCALARS: [&str; 5] = ["Int", "Float", "Boolean", "String", "ID"];

pub fn check(db: &dyn ValidationDatabase) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = Vec::new();

    for (name, scalar) in db.scalars().iter() {
        if let Some(loc) = scalar.loc() {
            // All built-in scalars must be omitted for brevity.
            if BUILT_IN_SCALARS.contains(&&**name) && !scalar.is_built_in() {
                diagnostics.push(
                    ApolloDiagnostic::new(db, loc.into(), DiagnosticData::BuiltInScalarDefinition)
                        .label(Label::new(loc, "remove this scalar definition")),
                );
            } else if !scalar.is_built_in() {
                // Custom scalars must provide a scalar specification URL via the
                // @specifiedBy directive
                if !scalar
                    .directives()
                    .iter()
                    .any(|directive| directive.name() == "specifiedBy")
                {
                    diagnostics.push(
                        ApolloDiagnostic::new(
                            db,
                            loc.into(),
                            DiagnosticData::ScalarSpecificationURL,
                        )
                        .label(Label::new(
                            loc,
                            "consider adding a @specifiedBy directive to this scalar definition",
                        )),
                    )
                }
            }
        }
    }

    diagnostics
}
