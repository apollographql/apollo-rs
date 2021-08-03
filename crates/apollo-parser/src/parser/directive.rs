use crate::{argument, input_value_definition, name, Parser, SyntaxKind, TokenKind};

use crate::format_err;

/// See: https://spec.graphql.org/June2018/#DirectiveDefinition
///
/// ```txt
/// DirectiveDefinition
///     Description(opt) directive @ Name ArgumentsDefinition(opt) on DirectiveLocations
/// ```
pub(crate) fn directive_definition(parser: &mut Parser) -> Result<(), crate::Error> {
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
        input_value_definition(parser, false)?;
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
        Some(TokenKind::Node) => {
            match parser.peek_data() {
                Some(loc) => {
                    let _guard = parser.start_node(SyntaxKind::DIRECTIVE_LOCATION);
                    match loc.as_str() {
                        "MUTATION" => parser.bump(SyntaxKind::QUERY_KW),
                        "SUBSCRIPTION" => parser.bump(SyntaxKind::SUBSCRIPTION_KW),
                        "FIELD" => parser.bump(SyntaxKind::FIELD_KW),
                        "FRAGMENT_DEFINITION" => parser.bump(SyntaxKind::FRAGMENT_DEFINITION_KW),
                        "FRAGMENT_SPREAD" => parser.bump(SyntaxKind::FRAGMENT_DEFINITION_KW),
                        "INLINE_FRAGMENT" => parser.bump(SyntaxKind::INLINE_FRAGMENT_KW),
                        "SCHEMA" => parser.bump(SyntaxKind::SCHEMA_KW),
                        "SCALAR" => parser.bump(SyntaxKind::SCALAR_KW),
                        "OBJECT" => parser.bump(SyntaxKind::OBJECT_KW),
                        "FIELD_DEFINITION" => parser.bump(SyntaxKind::FIELD_DEFINITION_KW),
                        "ARGUMENT_DEFINITION" => parser.bump(SyntaxKind::ARGUMENT_DEFINITION_KW),
                        "INTERFACE" => parser.bump(SyntaxKind::INTERFACE_KW),
                        "UNION" => parser.bump(SyntaxKind::UNION_KW),
                        "ENUM" => parser.bump(SyntaxKind::ENUM_KW),
                        "ENUM_VALUE" => parser.bump(SyntaxKind::ENUM_VALUE_KW),
                        "INPUT_OBJECT" => parser.bump(SyntaxKind::INPUT_OBJECT_KW),
                        "INPUT_FIELD_DEFINITION" => {
                            parser.bump(SyntaxKind::INPUT_FIELD_DEFINITION_KW)
                        }
                        _ => todo!(),
                    }
                }
                None => todo!(),
            }
            match parser.peek_data() {
                Some(_) => directive_locations(parser, true),
                _ => Ok(()),
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

/// See: https://spec.graphql.org/June2018/#Directive
///
/// ```txt
/// Directive
///     @ Name Arguments
/// ```
pub(crate) fn directive(parser: &mut Parser) -> Result<(), crate::Error> {
    let _guard = parser.start_node(SyntaxKind::DIRECTIVE);

    match parser.peek() {
        Some(TokenKind::At) => parser.bump(SyntaxKind::AT),
        _ => {
            return format_err!(
                parser.peek_data().unwrap(),
                "Expected directive @ name, got {}",
                parser.peek_data().unwrap()
            );
        }
    }

    name(parser)?;

    if let Some(TokenKind::LParen) = parser.peek() {
        let guard = parser.start_node(SyntaxKind::ARGUMENTS);
        parser.bump(SyntaxKind::L_PAREN);
        argument(parser, false)?;
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

    Ok(())
}
