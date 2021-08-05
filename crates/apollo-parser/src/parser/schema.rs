use crate::{directive, format_err, operation_type_definition, Parser, SyntaxKind, TokenKind};

/// See: https://spec.graphql.org/June2018/#SchemaDefinition
///
/// ```txt
/// SchemaDefinition
///    schema Directives { OperationTypeDefinition }
/// ```
pub(crate) fn schema_definition(parser: &mut Parser) -> Result<(), crate::Error> {
    let _guard = parser.start_node(SyntaxKind::SCHEMA_DEFINITION);

    parser.bump(SyntaxKind::schema_KW);
    directive(parser)?;

    if let Some(TokenKind::LCurly) = parser.peek() {
        parser.bump(SyntaxKind::L_CURLY);
        operation_type_definition(parser)?;
        if let Some(TokenKind::RCurly) = parser.peek() {
            parser.bump(SyntaxKind::R_CURLY);
        } else {
            return format_err!(
                parser.peek_data().unwrap(),
                "Expected Schema Definition to have a closing curly bracket, got {}",
                parser.peek_data().unwrap()
            );
        }
    } else {
        return format_err!(
            parser.peek_data().unwrap(),
            "Expected Schema Definition to define a root operation, got {}",
            parser.peek_data().unwrap()
        );
    }

    Ok(())
}
