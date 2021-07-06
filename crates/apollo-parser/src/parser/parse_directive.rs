use crate::{parse_directive_locations, Parser, SyntaxKind, TokenKind};

use super::parse_input_value_definitions;

/// See: https://spec.graphql.org/June2018/#DirectiveDefinition
///
/// ```txt
/// DirectiveDefinition
///     Description(opt) directive @ Name ArgumentsDefinition(opt) on DirectiveLocations
/// ```
pub(crate) fn parse_directive(parser: &mut Parser) -> Result<(), ()> {
    parser.builder.start_node(SyntaxKind::DIRECTIVE_DEFINITION);
    // TODO lrlna: parse Description
    parser.bump(SyntaxKind::directive_KW);
    // parser.parse_whitespace();

    match parser.peek() {
        Some(TokenKind::At) => parser.bump(SyntaxKind::AT),
        // missing directive name
        _ => return Err(()),
    }
    match parser.peek() {
        // TODO lrlna: use parse name function
        Some(TokenKind::Node) => parser.bump(SyntaxKind::NAME),
        // missing directive name
        _ => return Err(()),
    }

    match parser.peek() {
        Some(TokenKind::LParen) => {
            parser.bump(SyntaxKind::L_PAREN);
            parse_input_value_definitions(parser, false)?;
            match parser.peek() {
                Some(TokenKind::RParen) => parser.bump(SyntaxKind::R_PAREN),
                // missing a closing RParen
                _ => return Err(()),
            }

            match parser.peek() {
                Some(TokenKind::On) => parser.bump(SyntaxKind::on_KW),
                // missing directive locations in directive definition
                _ => return Err(()),
            }
        }
        Some(TokenKind::On) => parser.bump(SyntaxKind::on_KW),
        // missing directive locations in directive definition
        _ => return Err(()),
    }

    parse_directive_locations(parser, false)?;
    parser.builder.finish_node();
    Ok(())
}
