use crate::{ast, schema, validation::diagnostics::ValidationError, Node, ValidationDatabase};

pub(crate) fn validate_scalar_definitions(db: &dyn ValidationDatabase) -> Vec<ValidationError> {
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
) -> Vec<ValidationError> {
    let mut diagnostics = Vec::new();

    // All built-in scalars must be omitted for brevity.
    if !scalar_def.is_built_in() {
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
