use crate::{
    diagnostics::{BuiltInScalarDefinition, ScalarSpecificationURL},
    ApolloDiagnostic, ValidationDatabase,
};

const BUILT_IN_SCALARS: [&str; 5] = ["Int", "Float", "Boolean", "String", "ID"];

pub fn check(db: &dyn ValidationDatabase) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = Vec::new();

    for scalar in db.scalars().iter() {
        let name = scalar.name();
        if let Some(node) = scalar.ast_node(db.upcast()) {
            let offset: usize = node.text_range().start().into();
            let len: usize = node.text_range().len().into();

            // All built-in scalars must be omitted for brevity.
            if BUILT_IN_SCALARS.contains(&name) && !scalar.is_built_in() {
                diagnostics.push(ApolloDiagnostic::BuiltInScalarDefinition(
                    BuiltInScalarDefinition {
                        scalar: (offset, len).into(),
                        src: db.input_document(),
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
                    diagnostics.push(ApolloDiagnostic::ScalarSpecificationURL(
                        ScalarSpecificationURL {
                            scalar: (offset, len).into(),
                            src: db.input_document(),
                        },
                    ))
                }
            }
        }
    }

    diagnostics
}
