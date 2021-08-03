use crate::{directive, Parser, SyntaxKind};

/// See: https://spec.graphql.org/June2018/#SchemaDefinition
///
/// ```txt
/// SchemaDefinition
///    schema Directives { RootOperationTypeDefinition }
/// ```
pub(crate) fn schema_definition(parser: &mut Parser) -> Result<(), crate::Error> {
    let _guard = parser.start_node(SyntaxKind::SCHEMA_DEFINITION);
    // TODO lrlna: parse description???
    parser.bump(SyntaxKind::schema_KW);
    directive(parser)?;

    Ok(())
}
