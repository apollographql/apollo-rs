use crate::{diagnostics::ErrorDiagnostic, ApolloDiagnostic, SourceDatabase};

const BUILT_IN_SCALARS: [&str; 5] = ["Int", "Float", "Boolean", "String", "ID"];

pub fn check(db: &dyn SourceDatabase) -> Vec<ApolloDiagnostic> {
    let mut errors = Vec::new();

    // All built-in scalars must be omitted for brevity.
    for scalar in db.scalars().iter() {
        let name = scalar.name();
        if BUILT_IN_SCALARS.contains(&name) {
            errors.push(ApolloDiagnostic::Error(
                ErrorDiagnostic::BuiltInScalarDefinition {
                    message: "Built-in scalars must be omitted for brevity".into(),
                    scalar: name.into(),
                },
            ));
        } else {
            // Custom scalars must provide a scalar specification URL via the
            // @specifiedBy directive
            if !scalar
                .directives()
                .iter()
                .any(|directive| directive.name() == "specifiedBy")
            {
                errors.push(ApolloDiagnostic::Error(
                ErrorDiagnostic::ScalarSpecificationURL {
                    message: "Custom scalars must provide a scalar specification URL via the @specifiedBy directive".into(),
                    scalar: name.into(),
                },
            ));
            }
        }
    }

    errors
}
