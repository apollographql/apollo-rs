use crate::{ast, schema, validation::diagnostics::ValidationError, Node};

pub(crate) fn validate_scalar_definitions(schema: &crate::Schema) -> Vec<ValidationError> {
    let mut diagnostics = Vec::new();

    for def in schema.types.values() {
        if let schema::ExtendedType::Scalar(scalar) = def {
            diagnostics.extend(validate_scalar_definition(schema, scalar));
        }
    }

    diagnostics
}

pub(crate) fn validate_scalar_definition(
    schema: &crate::Schema,
    scalar_def: &Node<schema::ScalarType>,
) -> Vec<ValidationError> {
    let mut diagnostics = Vec::new();

    // All built-in scalars must be omitted for brevity.
    if !scalar_def.is_built_in() {
        diagnostics.extend(super::directive::validate_directives(
            Some(schema),
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
