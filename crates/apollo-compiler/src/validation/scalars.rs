use crate::{
    diagnostics::{BuiltInScalarDefinition, ScalarSpecificationURL},
    ApolloDiagnostic, SourceDatabase,
};

const BUILT_IN_SCALARS: [&str; 5] = ["Int", "Float", "Boolean", "String", "ID"];

pub fn check(db: &dyn SourceDatabase) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = Vec::new();

    // All built-in scalars must be omitted for brevity.
    for scalar in db.scalars().iter() {
        let name = scalar.name();
        let offset: usize = scalar.ast_node(db).text_range().start().into();
        let len: usize = scalar.ast_node(db).text_range().len().into();
        if BUILT_IN_SCALARS.contains(&name) {
            diagnostics.push(ApolloDiagnostic::BuiltInScalarDefinition(
                BuiltInScalarDefinition {
                    scalar: (offset, len).into(),
                    src: db.input_string(()).to_string(),
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
                diagnostics.push(ApolloDiagnostic::ScalarSpecificationURL(
                    ScalarSpecificationURL {
                        scalar: (offset, len).into(),
                        src: db.input_string(()).to_string(),
                    },
                ));
            }
        }
    }

    diagnostics
}