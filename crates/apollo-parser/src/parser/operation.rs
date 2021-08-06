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
pub(crate) fn operation_type_definition(
    parser: &mut Parser,
    is_operation_type: bool,
) -> Result<(), crate::Error> {
    if let Some(TokenKind::Comma) = parser.peek() {
        parser.bump(SyntaxKind::COMMA);
        return operation_type_definition(parser, is_operation_type);
    }

    if let Some(TokenKind::Node) = parser.peek() {
        let guard = parser.start_node(SyntaxKind::OPERATION_TYPE_DEFINITION);
        operation_type(parser)?;
        if let Some(TokenKind::Colon) = parser.peek() {
            parser.bump(SyntaxKind::COLON);
            named_type(parser)?;
            if parser.peek().is_some() {
                guard.finish_node();
                return operation_type_definition(parser, true);
            }
            return Ok(());
        } else {
            return format_err!(
                parser.peek_data().unwrap(),
                "Expected Operation Type to have a Name Type, got {}",
                parser.peek_data().unwrap()
            );
        }
    }

    if is_operation_type {
        Ok(())
    } else {
        return format_err!(
            parser.peek_data().unwrap(),
            "Expected Schema Definition to have an Operation Type, got {}",
            parser.peek_data().unwrap()
        );
    }
}

/// See: https://spec.graphql.org/June2018/#OperationType
///
/// ```txt
/// OperationType : one of
///    query    mutation    subscription
/// ```
pub(crate) fn operation_type(parser: &mut Parser) -> Result<(), crate::Error> {
    if let Some(node) = parser.peek_data() {
        let _guard = parser.start_node(SyntaxKind::OPERATION_TYPE);
        match node.as_str() {
            "query" => parser.bump(SyntaxKind::query_KW),
            "subscription" => parser.bump(SyntaxKind::subscription_KW),
            "mutation" => parser.bump(SyntaxKind::mutation_KW),
            _ => {
                return format_err!(
                    parser.peek_data().unwrap(),
                    "Operation Type must be either 'mutation', 'query' or 'subscription', got {}",
                    parser.peek_data().unwrap()
                )
            }
        }
    }

    Ok(())
}
