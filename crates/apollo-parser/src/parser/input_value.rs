use crate::{format_err, parser::name, Parser, SyntaxKind, TokenKind};

/// See: https://spec.graphql.org/June2018/#InputValueDefinition
///
/// ```txt
/// InputValueDefinition
///     Description(opt) Name : Type DefaultValue(opt) Directives(const/opt)
/// ```
pub(crate) fn input_value_definition(
    parser: &mut Parser,
    is_input: bool,
) -> Result<(), crate::Error> {
    // TODO @lrlna: parse description
    // TODO @lrlna: parse default value
    // TODO @lrlna: parse directives
    if let Some(TokenKind::Node) = parser.peek() {
        // TODO @lrlna: use parse input value name function
        let guard = parser.start_node(SyntaxKind::INPUT_VALUE_DEFINITION);
        name::name(parser)?;
        if let Some(TokenKind::Colon) = parser.peek() {
            parser.bump(SyntaxKind::COLON);
            if let Some(TokenKind::Node) = parser.peek() {
                // TODO @lrlna: type is a node, and needs its own parsing rules
                parser.bump(SyntaxKind::TYPE);
                if parser.peek().is_some() {
                    guard.finish_node();
                    return input_value_definition(parser, true);
                }
                return Ok(());
            }
            return format_err!(
                parser.peek_data().unwrap(),
                "Expected InputValue definition to have a Type, got {}",
                parser.peek_data().unwrap()
            );
        } else {
            return format_err!(
                parser.peek_data().unwrap(),
                "Expected InputValue definition to have a Name, got {}",
                parser.peek_data().unwrap()
            );
        }
    }
    if let Some(TokenKind::Comma) = parser.peek() {
        parser.bump(SyntaxKind::COMMA);
        return input_value_definition(parser, is_input);
    }
    // if we already have an input, can proceed without returning an error
    if is_input {
        Ok(())
    } else {
        // if there is no input, and a LPAREN was supplied, send an error
        return format_err!(
            parser.peek_data().unwrap(),
            "Expected to have an InputValue definition, got {}",
            parser.peek_data().unwrap()
        );
    }
}
