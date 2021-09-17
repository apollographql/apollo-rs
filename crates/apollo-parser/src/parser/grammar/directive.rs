use crate::parser::grammar::{argument, input, name};
use crate::{create_err, Parser, SyntaxKind, TokenKind, S, T};

/// See: https://spec.graphql.org/June2018/#DirectiveDefinition
///
/// ```txt
/// DirectiveDefinition
///     Description(opt) directive @ Name ArgumentsDefinition(opt) repeatable(opt) on DirectiveLocations
/// ```
pub(crate) fn directive_definition(p: &mut Parser) {
    let _guard = p.start_node(SyntaxKind::DIRECTIVE_DEFINITION);
    // TODO @lrlna: parse Description
    p.bump(SyntaxKind::directive_KW);
    match p.peek() {
        Some(T![@]) => p.bump(S![@]),
        _ => {
            p.push_err(create_err!(
                p.peek_data().unwrap(),
                "Expected directive @ definition, got {}",
                p.peek_data().unwrap()
            ));
        }
    }
    name::name(p);

    if let Some(T!['(']) = p.peek() {
        let guard = p.start_node(SyntaxKind::ARGUMENTS_DEFINITION);
        p.bump(S!['(']);
        input::input_value_definition(p, false);
        match p.peek() {
            Some(T![')']) => {
                p.bump(S![')']);
                guard.finish_node();
            }
            _ => p.push_err(create_err!(
                p.peek_data()
                    .unwrap_or_else(|| String::from("no further data")),
                "Expected closing ')', got {}",
                p.peek_data()
                    .unwrap_or_else(|| String::from("no further data"))
            )),
        }
    }

    if let Some(node) = p.peek_data() {
        if node.as_str() == "repeatable" {
            p.bump(SyntaxKind::repeatable_KW);
        }
    }

    if let Some(node) = p.peek_data() {
        match node.as_str() {
            "on" => p.bump(SyntaxKind::on_KW),
            _ => p.push_err(create_err!(
                p.peek_data()
                    .unwrap_or_else(|| String::from("no further data")),
                "Expected to have Directive Locations in a Directive Definition, got {}",
                p.peek_data()
                    .unwrap_or_else(|| String::from("no further data"))
            )),
        }
    }

    if let Some(TokenKind::Name | T![|]) = p.peek() {
        let _guard = p.start_node(SyntaxKind::DIRECTIVE_LOCATIONS);
        directive_locations(p, false);
    } else {
        p.push_err(create_err!(
            p.peek_data()
                .unwrap_or_else(|| String::from("no further data")),
            "Expected to have a valid Directive Location in a Directive Definition, got {}",
            p.peek_data()
                .unwrap_or_else(|| String::from("no further data"))
        ));
    }
}

/// See: https://spec.graphql.org/June2018/#DirectiveLocations
pub(crate) fn directive_locations(p: &mut Parser, is_location: bool) {
    if let Some(T![|]) = p.peek() {
        p.bump(S![|]);
        directive_locations(p, is_location)
    }

    if let Some(TokenKind::Name) = p.peek() {
        let loc = p.peek_data().unwrap();
        match loc.as_str() {
            "MUTATION" => {
                let _guard = p.start_node(SyntaxKind::DIRECTIVE_LOCATION);
                p.bump(SyntaxKind::QUERY_KW);
            }
            "SUBSCRIPTION" => {
                let _guard = p.start_node(SyntaxKind::DIRECTIVE_LOCATION);
                p.bump(SyntaxKind::SUBSCRIPTION_KW);
            }
            "FIELD" => {
                let _guard = p.start_node(SyntaxKind::DIRECTIVE_LOCATION);
                p.bump(SyntaxKind::FIELD_KW);
            }
            "FRAGMENT_DEFINITION" => {
                let _guard = p.start_node(SyntaxKind::DIRECTIVE_LOCATION);
                p.bump(SyntaxKind::FRAGMENT_DEFINITION_KW);
            }
            "FRAGMENT_SPREAD" => {
                let _guard = p.start_node(SyntaxKind::DIRECTIVE_LOCATION);
                p.bump(SyntaxKind::FRAGMENT_DEFINITION_KW);
            }
            "INLINE_FRAGMENT" => {
                let _guard = p.start_node(SyntaxKind::DIRECTIVE_LOCATION);
                p.bump(SyntaxKind::INLINE_FRAGMENT_KW);
            }
            "SCHEMA" => {
                let _guard = p.start_node(SyntaxKind::DIRECTIVE_LOCATION);
                p.bump(SyntaxKind::SCHEMA_KW);
            }
            "SCALAR" => {
                let _guard = p.start_node(SyntaxKind::DIRECTIVE_LOCATION);
                p.bump(SyntaxKind::SCALAR_KW);
            }
            "OBJECT" => {
                let _guard = p.start_node(SyntaxKind::DIRECTIVE_LOCATION);
                p.bump(SyntaxKind::OBJECT_KW);
            }
            "FIELD_DEFINITION" => {
                let _guard = p.start_node(SyntaxKind::DIRECTIVE_LOCATION);
                p.bump(SyntaxKind::FIELD_DEFINITION_KW);
            }
            "ARGUMENT_DEFINITION" => {
                let _guard = p.start_node(SyntaxKind::DIRECTIVE_LOCATION);
                p.bump(SyntaxKind::ARGUMENT_DEFINITION_KW);
            }
            "INTERFACE" => {
                let _guard = p.start_node(SyntaxKind::DIRECTIVE_LOCATION);
                p.bump(SyntaxKind::INTERFACE_KW);
            }
            "UNION" => {
                let _guard = p.start_node(SyntaxKind::DIRECTIVE_LOCATION);
                p.bump(SyntaxKind::UNION_KW);
            }
            "ENUM" => {
                let _guard = p.start_node(SyntaxKind::DIRECTIVE_LOCATION);
                p.bump(SyntaxKind::ENUM_KW);
            }
            "ENUM_VALUE" => {
                let _guard = p.start_node(SyntaxKind::DIRECTIVE_LOCATION);
                p.bump(SyntaxKind::ENUM_VALUE_KW);
            }
            "INPUT_OBJECT" => {
                let _guard = p.start_node(SyntaxKind::DIRECTIVE_LOCATION);
                p.bump(SyntaxKind::INPUT_OBJECT_KW);
            }
            "INPUT_FIELD_DEFINITION" => {
                let _guard = p.start_node(SyntaxKind::DIRECTIVE_LOCATION);
                p.bump(SyntaxKind::INPUT_FIELD_DEFINITION_KW);
            }
            _ => {
                if !is_location {
                    p.push_err(create_err!(
                        p
                            .peek_data()
                            .unwrap_or_else(|| String::from("no further data")),
                        "Expected to have a valid Directive Location in a Directive Definition, got {}",
                        p
                            .peek_data()
                            .unwrap_or_else(|| String::from("no further data"))
                    ));
                }
                return;
            }
        }
        if p.peek_data().is_some() {
            return directive_locations(p, true);
        }
    }
    if !is_location {
        p.push_err(create_err!(
            p.peek_data()
                .unwrap_or_else(|| String::from("no further data")),
            "Expected to have Directive Locations in a Directive Definition, got {}",
            p.peek_data()
                .unwrap_or_else(|| String::from("no further data"))
        ));
    }
}

/// See: https://spec.graphql.org/June2018/#Directive
///
/// ```txt
/// Directive
///     @ Name Arguments
/// ```
pub(crate) fn directive(p: &mut Parser) {
    let _guard = p.start_node(SyntaxKind::DIRECTIVE);

    match p.peek() {
        Some(T![@]) => p.bump(S![@]),
        _ => {
            p.push_err(create_err!(
                p.peek_data().unwrap(),
                "Expected directive @ definition, got {}",
                p.peek_data().unwrap()
            ));
        }
    }

    name::name(p);

    if let Some(T!['(']) = p.peek() {
        argument::arguments(p);
    }
}

pub(crate) fn directives(p: &mut Parser) {
    let _guard = p.start_node(SyntaxKind::DIRECTIVES);
    while let Some(T![@]) = p.peek() {
        directive(p);
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
            - ERROR@0:15 "Expected to have Directive Locations in a Directive Definition, got no further data"
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
                        - DIRECTIVE_LOCATION@40..54
                            - FIELD_KW@40..45 "FIELD"
                            - PIPE@45..46 "|"
                            - DIRECTIVE_LOCATION@46..54
                                - QUERY_KW@46..54 "MUTATION"
            "#,
        );
    }

    // TODO @lrlna: enable the "repeatable" graphql extension
    //
    // See: https://spec.graphql.org/draft/#sec-Type-System.Directives
    #[test]
    fn it_parses_repeatable_nodes() {
        utils::check_ast(
            "directive @example(isTreat: Boolean, treatKind: String) repeatable on FIELD | MUTATION",
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
                        - DIRECTIVE_LOCATION@40..54
                            - FIELD_KW@40..45 "FIELD"
                            - PIPE@45..46 "|"
                            - DIRECTIVE_LOCATION@46..54
                                - QUERY_KW@46..54 "MUTATION"
            "#,
        );
    }
}
