use crate::{input_value_definitions, name, Parser, SyntaxKind, TokenKind};

use crate::format_err;

/// See: https://spec.graphql.org/June2018/#DirectiveDefinition
///
/// ```txt
/// DirectiveDefinition
///     Description(opt) directive @ Name ArgumentsDefinition(opt) on DirectiveLocations
/// ```
pub(crate) fn directive(parser: &mut Parser) -> Result<(), crate::Error> {
    let _guard = parser.start_node(SyntaxKind::DIRECTIVE_DEFINITION);
    // TODO lrlna: parse Description
    parser.bump(SyntaxKind::directive_KW);
    // parser.parse_whitespace();

    match parser.peek() {
        Some(TokenKind::At) => parser.bump(SyntaxKind::AT),
        // missing directive name
        _ => {
            return format_err!(
                parser.peek_data().unwrap(),
                "Expected directive @ definition, got {}",
                parser.peek_data().unwrap()
            );
        }
    }
    name(parser)?;

    if let Some(TokenKind::LParen) = parser.peek() {
        let guard = parser.start_node(SyntaxKind::ARGUMENTS_DEFINITION);
        parser.bump(SyntaxKind::L_PAREN);
        input_value_definitions(parser, false)?;
        match parser.peek() {
            Some(TokenKind::RParen) => {
                parser.bump(SyntaxKind::R_PAREN);
                guard.finish_node();
            }
            // missing a closing RParen
            _ => {
                return format_err!(
                    parser.peek_data().unwrap(),
                    "Expected closing ')', got {}",
                    parser.peek_data().unwrap()
                )
            }
        }
    }

    match parser.peek() {
        Some(TokenKind::On) => parser.bump(SyntaxKind::on_KW),
        // missing directive locations in directive definition
        _ => {
            return format_err!(
                parser.peek_data().unwrap(),
                "Expected to have Directive locations in a directive definition, got {}",
                parser.peek_data().unwrap()
            )
        }
    }

    let _guard = parser.start_node(SyntaxKind::DIRECTIVE_LOCATIONS);
    directive_locations(parser, false)?;
    Ok(())
}

/// See: https://spec.graphql.org/June2018/#DirectiveLocations
pub(crate) fn directive_locations(
    parser: &mut Parser,
    is_location: bool,
) -> Result<(), crate::Error> {
    match parser.peek() {
        Some(TokenKind::Pipe) => {
            parser.bump(SyntaxKind::PIPE);
            directive_locations(parser, is_location)
        }
        // TODO lrlna: Syntax Kind here is wrong. This should match on either
        // TypeSystemDirectiveLocation or ExecutableDirectiveLocation.
        Some(TokenKind::Node) => {
            parser.bump(SyntaxKind::DIRECTIVE_LOCATION);
            match parser.peek_data() {
                Some(_) => return directive_locations(parser, true),
                _ => return Ok(()),
            }
        }
        _ => {
            if !is_location {
                // missing directive locations in directive definition
                return format_err!(
                    parser.peek_data().unwrap(),
                    "Expected to have Directive locations in a directive definition, got {}",
                    parser.peek_data().unwrap()
                );
            }
            Ok(())
        }
    }
}
