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
    // TODO @lrlna: parse Description
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
                    parser
                        .peek_data()
                        .unwrap_or_else(|| String::from("no further data")),
                    "Expected closing ')', got {}",
                    parser
                        .peek_data()
                        .unwrap_or_else(|| String::from("no further data"))
                )
            }
        }
    }

    match parser.peek() {
        Some(TokenKind::On) => parser.bump(SyntaxKind::on_KW),
        // missing directive locations in directive definition
        _ => {
            return format_err!(
                parser
                    .peek_data()
                    .unwrap_or_else(|| String::from("no further data")),
                "Expected to have Directive locations in a directive definition, got {}",
                parser
                    .peek_data()
                    .unwrap_or_else(|| String::from("no further data"))
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
                    parser
                        .peek_data()
                        .unwrap_or_else(|| String::from("no further data")),
                    "Expected to have Directive locations in a directive definition, got {}",
                    parser
                        .peek_data()
                        .unwrap_or_else(|| String::from("no further data"))
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
                    parser
                        .peek_data()
                        .unwrap_or_else(|| String::from("no further data")),
                    "Expected closing ')', got {}",
                    parser
                        .peek_data()
                        .unwrap_or_else(|| String::from("no further data"))
                )
            }
        }
    }

    Ok(())
}

// TODO @lrlna: inlined collapsed AST should live in a 'fixtures' dir for ease of testing
#[cfg(test)]
mod test {
    use super::*;
    use indoc::indoc;
    use pretty_assertions::assert_eq;

    #[test]
    fn smoke_directive_definition() {
        let input = "directive @example(isTreat: Boolean, treatKind: String) on FIELD | MUTATION";
        let parser = Parser::new(input);

        println!("{:?}", parser.parse());
    }

    // TODO @lrlna: these tests need to check for indentation as part of the
    // output, not just the nodes of the tree
    #[test]
    fn it_parses_directive_definition() {
        let input = "directive @example(isTreat: Boolean, treatKind: String) on FIELD | MUTATION";
        let parser = Parser::new(input);
        let output = parser.parse();

        assert!(output.errors().is_empty());
        assert_eq!(
            format!("{:?}", output),
            indoc! { r#"
            - DOCUMENT@0..67
            - DIRECTIVE_DEFINITION@0..67
            - directive_KW@0..9 "directive"
            - AT@9..10 "@"
            - NAME@10..17
            - IDENT@10..17 "example"
            - ARGUMENTS_DEFINITION@17..51
            - L_PAREN@17..18 "("
            - INPUT_VALUE_DEFINITION@18..33
            - NAME@18..25
            - IDENT@18..25 "isTreat"
            - COLON@25..26 ":"
            - TYPE@26..33 "Boolean"
            - COMMA@33..34 ","
            - INPUT_VALUE_DEFINITION@34..50
            - NAME@34..43
            - IDENT@34..43 "treatKind"
            - COLON@43..44 ":"
            - TYPE@44..50 "String"
            - R_PAREN@50..51 ")"
            - on_KW@51..53 "on"
            - DIRECTIVE_LOCATIONS@53..67
            - DIRECTIVE_LOCATION@53..58
            - FIELD_KW@53..58 "FIELD"
            - PIPE@58..59 "|"
            - DIRECTIVE_LOCATION@59..67
            - QUERY_KW@59..67 "MUTATION"
            "# }
        );
    }
}
