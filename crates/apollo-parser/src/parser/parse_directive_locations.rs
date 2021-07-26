use crate::format_err;
use crate::{Parser, SyntaxKind, TokenKind};

/// See: https://spec.graphql.org/June2018/#DirectiveLocations
pub(crate) fn parse_directive_locations(
    parser: &mut Parser,
    is_location: bool,
) -> Result<(), crate::Error> {
    match parser.peek() {
        Some(TokenKind::Pipe) => {
            parser.bump(SyntaxKind::PIPE);
            parse_directive_locations(parser, is_location)
        }
        // TODO lrlna: Syntax Kind here is wrong. This should match on either
        // TypeSystemDirectiveLocation or ExecutableDirectiveLocation.
        Some(TokenKind::Node) => {
            parser.bump(SyntaxKind::DIRECTIVE_LOCATION);
            match parser.peek_data() {
                Some(_) => return parse_directive_locations(parser, true),
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
