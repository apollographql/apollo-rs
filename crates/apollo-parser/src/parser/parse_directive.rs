use crate::{parse_directive_locations, parse_name, Parser, SyntaxKind, TokenKind};

use super::parse_input_value_definitions;

/// See: https://spec.graphql.org/June2018/#DirectiveDefinition
///
/// ```txt
/// DirectiveDefinition
///     Description(opt) directive @ Name ArgumentsDefinition(opt) on DirectiveLocations
/// ```
pub(crate) fn parse_directive(parser: &mut Parser) -> Result<(), ()> {
    let _guard = parser.start_node(SyntaxKind::DIRECTIVE_DEFINITION);
    // TODO lrlna: parse Description
    parser.bump(SyntaxKind::directive_KW);
    // parser.parse_whitespace();

    match parser.peek() {
        Some(TokenKind::At) => parser.bump(SyntaxKind::AT),
        // missing directive name
        _ => return Err(()),
    }
    parse_name(parser)?;

    if let Some(TokenKind::LParen) = parser.peek() {
        let guard = parser.start_node(SyntaxKind::ARGUMENTS_DEFINITION);
        parser.bump(SyntaxKind::L_PAREN);
        parse_input_value_definitions(parser, false)?;
        match parser.peek() {
            Some(TokenKind::RParen) => {
                parser.bump(SyntaxKind::R_PAREN);
                drop(guard);
            }
            // missing a closing RParen
            _ => return Err(()),
        }
    }

    match parser.peek() {
        Some(TokenKind::On) => parser.bump(SyntaxKind::on_KW),
        // missing directive locations in directive definition
        _ => return Err(()),
    }

    let _guard = parser.start_node(SyntaxKind::DIRECTIVE_LOCATIONS);
    parse_directive_locations(parser, false)?;
    Ok(())
}
