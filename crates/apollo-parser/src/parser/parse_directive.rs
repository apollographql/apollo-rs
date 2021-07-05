use crate::{Parser, TokenKind};

use super::parse_input_value_definitions;

/// See: https://spec.graphql.org/June2018/#DirectiveDefinition
///
/// ```txt
/// DirectiveDefinition
///     Description(opt) directive @ Name ArgumentsDefinition(opt) on DirectiveLocations
/// ```
pub(crate) fn parse_directive(parser: &mut Parser) -> Result<(), ()> {
    parser.builder.start_node(TokenKind::Directive.into());
    // TODO(lrlna): parse Description
    parser.bump();
    // parser.parse_whitespace();

    match parser.peek() {
        Some(TokenKind::At) => parser.bump(),
        // missing directive name
        _ => return Err(()),
    }
    match parser.peek() {
        Some(TokenKind::Node) => parser.bump(),
        // missing directive name
        _ => return Err(()),
    }

    match parser.peek() {
        Some(TokenKind::LParen) => {
            parser.bump();
            parse_input_value_definitions(parser, false)?;
            match parser.peek() {
                Some(TokenKind::RParen) => parser.bump(),
                // missing a closing RParen
                _ => return Err(()),
            }

            match parser.peek() {
                Some(TokenKind::On) => parser.bump(),
                // missing directive locations in directive definition
                _ => return Err(()),
            }
        }
        Some(TokenKind::On) => parser.bump(),
        // missing directive locations in directive definition
        _ => return Err(()),
    }

    parser.parse_directive_locations(false)?;
    parser.builder.finish_node();
    Ok(())
}
