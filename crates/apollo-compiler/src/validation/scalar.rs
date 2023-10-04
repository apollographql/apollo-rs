use crate::{
    ast,
    diagnostics::{ApolloDiagnostic, DiagnosticData, Label},
    schema, Node, ValidationDatabase,
};

pub(crate) fn validate_scalar_definitions(db: &dyn ValidationDatabase) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = Vec::new();

    let schema = db.schema();
    for def in schema.types.values() {
        if let schema::ExtendedType::Scalar(scalar) = def {
            diagnostics.extend(db.validate_scalar_definition(scalar.clone()));
        }
    }

    diagnostics
}

pub(crate) fn validate_scalar_definition(
    db: &dyn ValidationDatabase,
    scalar_def: Node<schema::ScalarType>,
) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = Vec::new();

    // All built-in scalars must be omitted for brevity.
    if !scalar_def.is_built_in() {
        // Custom scalars must provide a scalar specification URL via the
        // @specifiedBy directive
        let has_specified_by = scalar_def.directives.has("specifiedBy");
        if !has_specified_by {
            if let Some(location) = scalar_def.location() {
                diagnostics.push(
                    ApolloDiagnostic::new(db, location, DiagnosticData::ScalarSpecificationURL)
                        .label(Label::new(
                            location,
                            "consider adding a @specifiedBy directive to this scalar definition",
                        )),
                );
            }
        }

        diagnostics.extend(super::directive::validate_directives(
            db,
            scalar_def
                .directives
                .iter()
                .map(|component| &component.node),
            ast::DirectiveLocation::Scalar,
            // scalars don't use variables
            Default::default(),
        ));
    }

    diagnostics
}
