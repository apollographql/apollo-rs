use crate::{format_err, named_type, Parser, SyntaxKind, TokenKind};

/// OperationTypeDefinition is used in a SchemaDefinition. Not to be confused
/// with OperationDefinition.
///
/// See: https://spec.graphql.org/June2018/#RootOperationTypeDefinition
///
/// ```txt
/// OperationTypeDefinition
///    OperationType : NamedType
/// ```
pub(crate) fn operation_type_definition(parser: &mut Parser) -> Result<(), crate::Error> {
    let _guard = parser.start_node(SyntaxKind::OPERATION_TYPE_DEFINITION);
    if let Some(TokenKind::Node) = parser.peek() {
        parser.bump(SyntaxKind::OPERATION_TYPE);
        if let Some(TokenKind::Colon) = parser.peek() {
            parser.bump(SyntaxKind::COLON);
            named_type(parser)?
        } else {
            return format_err!(
                parser.peek_data().unwrap(),
                "Operation Type must be proceeded by Named Type, got {}",
                parser.peek_data().unwrap()
            );
        }
    } else {
        return format_err!(
            parser.peek_data().unwrap(),
            "Expected Schema Definition to have an Operation Type, got {}",
            parser.peek_data().unwrap()
        );
    }
    Ok(())
}
