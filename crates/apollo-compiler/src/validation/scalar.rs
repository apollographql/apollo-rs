use crate::ast;
use crate::schema;
use crate::validation::DiagnosticList;
use crate::Node;

pub(crate) fn validate_scalar_definition(
    diagnostics: &mut DiagnosticList,
    schema: &crate::Schema,
    scalar_def: &Node<schema::ScalarType>,
) {
    // All built-in scalars must be omitted for brevity.
    if !scalar_def.is_built_in() {
        super::directive::validate_directives(
            diagnostics,
            Some(schema),
            scalar_def
                .directives
                .iter()
                .map(|component| &component.node),
            ast::DirectiveLocation::Scalar,
            // scalars don't use variables
            Default::default(),
        );
    }
}
