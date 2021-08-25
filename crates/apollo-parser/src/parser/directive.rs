use crate::parser::{argument, input_value, name};
use crate::{create_err, Parser, SyntaxKind, TokenKind};

/// See: https://spec.graphql.org/June2018/#DirectiveDefinition
///
/// ```txt
/// DirectiveDefinition
///     Description(opt) directive @ Name ArgumentsDefinition(opt) on DirectiveLocations
/// ```
pub(crate) fn directive_definition(parser: &mut Parser) {
    let _guard = parser.start_node(SyntaxKind::DIRECTIVE_DEFINITION);
    // TODO @lrlna: parse Description
    parser.bump(SyntaxKind::directive_KW);
    match parser.peek() {
        Some(TokenKind::At) => parser.bump(SyntaxKind::AT),
        _ => {
            parser.push_err(create_err!(
                parser.peek_data().unwrap(),
                "Expected directive @ definition, got {}",
                parser.peek_data().unwrap()
            ));
        }
    }
    name::name(parser);

    if let Some(TokenKind::LParen) = parser.peek() {
        let guard = parser.start_node(SyntaxKind::ARGUMENTS_DEFINITION);
        parser.bump(SyntaxKind::L_PAREN);
        input_value::input_value_definition(parser, false);
        match parser.peek() {
            Some(TokenKind::RParen) => {
                parser.bump(SyntaxKind::R_PAREN);
                guard.finish_node();
            }
            _ => parser.push_err(create_err!(
                parser
                    .peek_data()
                    .unwrap_or_else(|| String::from("no further data")),
                "Expected closing ')', got {}",
                parser
                    .peek_data()
                    .unwrap_or_else(|| String::from("no further data"))
            )),
        }
    }

    if let Some(node) = parser.peek_data() {
        match node.as_str() {
            "on" => parser.bump(SyntaxKind::on_KW),
            _ => parser.push_err(create_err!(
                parser
                    .peek_data()
                    .unwrap_or_else(|| String::from("no further data")),
                "Expected to have Directive Locations in a Directive Definition, got {}",
                parser
                    .peek_data()
                    .unwrap_or_else(|| String::from("no further data"))
            )),
        }
    }

    let _guard = parser.start_node(SyntaxKind::DIRECTIVE_LOCATIONS);
    directive_locations(parser, false);
}

/// See: https://spec.graphql.org/June2018/#DirectiveLocations
pub(crate) fn directive_locations(parser: &mut Parser, is_location: bool) {
    match parser.peek() {
        Some(TokenKind::Pipe) => {
            parser.bump(SyntaxKind::PIPE);
            directive_locations(parser, is_location)
        }
        Some(TokenKind::Node) => {
            let loc = parser.peek_data().unwrap();
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
                "INPUT_FIELD_DEFINITION" => parser.bump(SyntaxKind::INPUT_FIELD_DEFINITION_KW),
                _ => {
                    parser.push_err(create_err!(
                        parser
                            .peek_data()
                            .unwrap_or_else(|| String::from("no further data")),
                        "Expected to have a valid Directive Location in a Directive Definition, got {}",
                        parser
                            .peek_data()
                            .unwrap_or_else(|| String::from("no further data"))
                    ));
                }
            }
            if parser.peek_data().is_some() {
                directive_locations(parser, true)
            }
        }
        _ => {
            if !is_location {
                parser.push_err(create_err!(
                    parser
                        .peek_data()
                        .unwrap_or_else(|| String::from("no further data")),
                    "Expected to have Directive Locations in a Directive Definition, got {}",
                    parser
                        .peek_data()
                        .unwrap_or_else(|| String::from("no further data"))
                ));
            }
        }
    }
}

/// See: https://spec.graphql.org/June2018/#Directive
///
/// ```txt
/// Directive
///     @ Name Arguments
/// ```
pub(crate) fn directive(parser: &mut Parser) {
    let _guard = parser.start_node(SyntaxKind::DIRECTIVE);

    match parser.peek() {
        Some(TokenKind::At) => parser.bump(SyntaxKind::AT),
        _ => {
            parser.push_err(create_err!(
                parser.peek_data().unwrap(),
                "Expected directive @ definition, got {}",
                parser.peek_data().unwrap()
            ));
        }
    }

    name::name(parser);

    if let Some(TokenKind::LParen) = parser.peek() {
        argument::arguments(parser);
    }
}

pub(crate) fn directives(parser: &mut Parser) {
    let _guard = parser.start_node(SyntaxKind::DIRECTIVES);
    while let Some(TokenKind::At) = parser.peek() {
        directive(parser);
    }
}

// TODO @lrlna: inlined collapsed AST should live in a 'fixtures' dir for ease of testing
#[cfg(test)]
mod test {
    use crate::parser::utils;

    #[test]
    fn it_returns_errors_and_full_ast_when_location_is_missing() {
        utils::check_ast(
            "directive @example on",
            r#"
            - DOCUMENT@0..19
                - DIRECTIVE_DEFINITION@0..19
                    - directive_KW@0..9 "directive"
                    - AT@9..10 "@"
                    - NAME@10..17
                        - IDENT@10..17 "example"
                    - on_KW@17..19 "on"
                    - DIRECTIVE_LOCATIONS@19..19
            - ERROR@0:15 "Expected to have Directive Locations in a Directive Definition, got no
            further data"
            "#,
        );
    }

    // TODO @lrlna: these tests need to check for indentation as part of the
    // output, not just the nodes of the tree
    #[test]
    fn it_parses_directive_definition() {
        utils::check_ast(
            "directive @example(isTreat: Boolean, treatKind: String) on FIELD | MUTATION",
            r#"
            - DOCUMENT@0..54
                - DIRECTIVE_DEFINITION@0..54
                    - directive_KW@0..9 "directive"
                    - AT@9..10 "@"
                    - NAME@10..17
                        - IDENT@10..17 "example"
                    - ARGUMENTS_DEFINITION@17..38
                        - L_PAREN@17..18 "("
                        - INPUT_VALUE_DEFINITION@18..26
                            - NAME@18..25
                                - IDENT@18..25 "isTreat"
                            - COLON@25..26 ":"
                            - TYPE@26..26
                                - NAMED_TYPE@26..26
                        - COMMA@26..27 ","
                        - INPUT_VALUE_DEFINITION@27..37
                            - NAME@27..36
                                - IDENT@27..36 "treatKind"
                            - COLON@36..37 ":"
                            - TYPE@37..37
                                - NAMED_TYPE@37..37
                        - R_PAREN@37..38 ")"
                    - on_KW@38..40 "on"
                    - DIRECTIVE_LOCATIONS@40..54
                        - DIRECTIVE_LOCATION@40..45
                            - FIELD_KW@40..45 "FIELD"
                        - PIPE@45..46 "|"
                        - DIRECTIVE_LOCATION@46..54
                            - QUERY_KW@46..54 "MUTATION"
"#,
        );
    }
}
