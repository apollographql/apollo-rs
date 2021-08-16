use crate::{format_err, name, Parser, SyntaxKind, TokenKind};

/// See: https://spec.graphql.org/June2018/#VariableDefinition
///
/// ```txt
/// VariableDefinition
///     Variable : Type DefaultValue(opt)
/// ```
pub(crate) fn variable_definition(
    parser: &mut Parser,
    is_variable: bool,
) -> Result<(), crate::Error> {
    // TODO @lrlna: parse optional default values
    if let Some(TokenKind::Dollar) = parser.peek() {
        let guard = parser.start_node(SyntaxKind::VARIABLE_DEFINITION);
        variable(parser)?;
        if let Some(TokenKind::Colon) = parser.peek() {
            parser.bump(SyntaxKind::COLON);
            if let Some(TokenKind::Node) = parser.peek() {
                // TODO @lrlna: type is a node, and needs its own parsing rules
                parser.bump(SyntaxKind::TYPE);
                if parser.peek().is_some() {
                    guard.finish_node();
                    return variable_definition(parser, true);
                }
                return Ok(());
            }
            return format_err!(
                parser.peek_data().unwrap(),
                "Expected Variable Definition to have a Type, got {}",
                parser.peek_data().unwrap()
            );
        } else {
            return format_err!(
                parser.peek_data().unwrap(),
                "Expected Variable Definition to have a Name, got {}",
                parser.peek_data().unwrap()
            );
        }
    }
    if let Some(TokenKind::Comma) = parser.peek() {
        parser.bump(SyntaxKind::COMMA);
        return variable_definition(parser, is_variable);
    }
    // if we already have a variable , can proceed without returning an error
    if is_variable {
        Ok(())
    } else {
        // if there is no input, and a LPAREN was supplied, send an error
        return format_err!(
            parser.peek_data().unwrap(),
            "Expected to have an Variable Definition, got {}",
            parser.peek_data().unwrap()
        );
    }
}

/// See: https://spec.graphql.org/June2018/#Variable
///
/// ```txt
/// Variable
///     $ Name
/// ```
pub(crate) fn variable(parser: &mut Parser) -> Result<(), crate::Error> {
    let _guard = parser.start_node(SyntaxKind::VARIABLE);
    parser.bump(SyntaxKind::DOLLAR);
    name(parser)?;
    Ok(())
}
