use crate::parser::{directive, name, selection, ty};
use crate::{create_err, Parser, SyntaxKind, TokenKind};

/// See: https://spec.graphql.org/June2018/#FragmentDefinition
///
/// ```txt
/// FragmentDefinition
///     fragment FragmentName TypeCondition Directives(opt) SelectionSet
/// ```
pub(crate) fn fragment_definition(parser: &mut Parser) {
    let _guard = parser.start_node(SyntaxKind::FRAGMENT_DEFINITION);
    parser.bump(SyntaxKind::fragment_KW);

    fragment_name(parser);
    type_condition(parser);

    if let Some(TokenKind::At) = parser.peek() {
        directive::directives(parser);
    }

    match parser.peek() {
        Some(TokenKind::LCurly) => selection::selection_set(parser),
        _ => {
            parser.push_err(create_err!(
                parser
                    .peek_data()
                    .unwrap_or_else(|| String::from("no further data")),
                "Expected Fragment Definition to have a Selection Set, got {}",
                parser
                    .peek_data()
                    .unwrap_or_else(|| String::from("no further data")),
            ));
        }
    }
}

/// See: https://spec.graphql.org/June2018/#FragmentName
///
/// ```txt
/// FragmentName
///     Name *but not* on
/// ```
pub(crate) fn fragment_name(parser: &mut Parser) {
    let _guard = parser.start_node(SyntaxKind::FRAGMENT_NAME);
    match parser.peek() {
        Some(TokenKind::Node) => {
            if parser.peek_data().unwrap() == "on" {
                parser.push_err(create_err!(
                    parser
                        .peek_data()
                        .unwrap_or_else(|| String::from("no further data")),
                    "Fragment Name cannot be 'on'",
                ));
            }
            name::name(parser)
        }
        _ => parser.push_err(create_err!(
            parser.peek_data().unwrap(),
            "Expected Fragment Name, got {}",
            parser.peek_data().unwrap()
        )),
    }
}

/// See: https://spec.graphql.org/June2018/#TypeCondition
///
/// ```txt
/// TypeCondition
///     on NamedType
/// ```
pub(crate) fn type_condition(parser: &mut Parser) {
    let _guard = parser.start_node(SyntaxKind::TYPE_CONDITION);
    match parser.peek() {
        Some(TokenKind::Node) => {
            if parser.peek_data().unwrap() == "on" {
                parser.bump(SyntaxKind::on_KW);
            } else {
                parser.push_err(create_err!(
                    parser
                        .peek_data()
                        .unwrap_or_else(|| String::from("no further data")),
                    "Expected Fragment's Type Condition to have 'on', got {}",
                    parser
                        .peek_data()
                        .unwrap_or_else(|| String::from("no further data")),
                ));
            }
            ty::named_type(parser)
        }
        _ => parser.push_err(create_err!(
            parser.peek_data().unwrap(),
            "Expected Type Condition in a Fragment, got {}",
            parser.peek_data().unwrap()
        )),
    }
}

/// See: https://spec.graphql.org/June2018/#InlineFragment
///
/// ```txt
/// InlineFragment
///     ... TypeCondition[opt] Directives[opt] SelectionSet
/// ```
pub(crate) fn inline_fragment(parser: &mut Parser) {
    let _guard = parser.start_node(SyntaxKind::INLINE_FRAGMENT);
    parser.bump(SyntaxKind::SPREAD);
    if let Some(TokenKind::Node) = parser.peek() {
        type_condition(parser);
    }
    if let Some(TokenKind::At) = parser.peek() {
        directive::directives(parser);
    }
    match parser.peek() {
        Some(TokenKind::LCurly) => selection::selection_set(parser),
        _ => {
            parser.push_err(create_err!(
                parser
                    .peek_data()
                    .unwrap_or_else(|| String::from("no further data")),
                "Expected Inline Fragment to have a Selection Set, got {}",
                parser
                    .peek_data()
                    .unwrap_or_else(|| String::from("no further data")),
            ));
        }
    }
}

/// See: https://spec.graphql.org/June2018/#FragmentSpread
///
/// ```txt
/// FragmentSpread
///     ... FragmentName Directives[opt]
/// ```
pub(crate) fn fragment_spread(parser: &mut Parser) {
    let _guard = parser.start_node(SyntaxKind::FRAGMENT_SPREAD);
    parser.bump(SyntaxKind::SPREAD);
    match parser.peek() {
        Some(TokenKind::Node) => {
            fragment_name(parser);
        }
        _ => {
            parser.push_err(create_err!(
                parser
                    .peek_data()
                    .unwrap_or_else(|| String::from("no further data")),
                "Expected Fragment Spread to have a Name, got {}",
                parser
                    .peek_data()
                    .unwrap_or_else(|| String::from("no further data")),
            ));
        }
    }

    if let Some(TokenKind::At) = parser.peek() {
        directive::directives(parser);
    }
}

#[cfg(test)]
mod test {
    use crate::parser::utils;

    #[test]
    fn it_parses_fragment_definition() {
        utils::check_ast(
            "fragment friendFields on User @example {
              id
              name
              profilePic(size: 50)
            }
            ",
            r#"
            - DOCUMENT@0..61
                - FRAGMENT_DEFINITION@0..61
                    - fragment_KW@0..8 "fragment"
                    - FRAGMENT_NAME@8..20
                        - NAME@8..20
                            - IDENT@8..20 "friendFields"
                    - TYPE_CONDITION@20..26
                        - on_KW@20..22 "on"
                        - NAMED_TYPE@22..26
                            - NAME@22..26
                                - IDENT@22..26 "User"
                    - DIRECTIVES@26..34
                        - DIRECTIVE@26..34
                            - AT@26..27 "@"
                            - NAME@27..34
                                - IDENT@27..34 "example"
                    - SELECTION_SET@34..61
                        - L_CURLY@34..35 "{"
                        - SELECTION@35..60
                            - FIELD@35..37
                                - NAME@35..37
                                    - IDENT@35..37 "id"
                            - FIELD@37..41
                                - NAME@37..41
                                    - IDENT@37..41 "name"
                            - FIELD@41..60
                                - NAME@41..51
                                    - IDENT@41..51 "profilePic"
                                - ARGUMENTS@51..60
                                    - L_PAREN@51..52 "("
                                    - ARGUMENT@52..59
                                        - NAME@52..56
                                            - IDENT@52..56 "size"
                                        - COLON@56..57 ":"
                                        - VALUE@57..59
                                            - INT_VALUE@57..59 "50"
                                    - R_PAREN@59..60 ")"
                        - R_CURLY@60..61 "}"
            "#,
        );
    }

    #[test]
    fn it_parses_fragment_definition_with_fragment_spread() {
        utils::check_ast(
            "fragment friendFields on User {
                id
                name
                ...standardProfilePic
            }",
            r#"
            - DOCUMENT@0..55
                - FRAGMENT_DEFINITION@0..55
                    - fragment_KW@0..8 "fragment"
                    - FRAGMENT_NAME@8..20
                        - NAME@8..20
                            - IDENT@8..20 "friendFields"
                    - TYPE_CONDITION@20..26
                        - on_KW@20..22 "on"
                        - NAMED_TYPE@22..26
                            - NAME@22..26
                                - IDENT@22..26 "User"
                    - SELECTION_SET@26..55
                        - L_CURLY@26..27 "{"
                        - SELECTION@27..54
                            - FIELD@27..29
                                - NAME@27..29
                                    - IDENT@27..29 "id"
                            - FIELD@29..33
                                - NAME@29..33
                                    - IDENT@29..33 "name"
                            - FRAGMENT_SPREAD@33..54
                                - SPREAD@33..36 "..."
                                - FRAGMENT_NAME@36..54
                                    - NAME@36..54
                                        - IDENT@36..54 "standardProfilePic"
                        - R_CURLY@54..55 "}"
            "#,
        );
    }

    #[test]
    fn it_returns_error_with_invalid_fragment_name() {
        utils::check_ast(
            "fragment on User @example {
              id
            }
            ",
            r#"
            - DOCUMENT@0..26
                - FRAGMENT_DEFINITION@0..26
                    - fragment_KW@0..8 "fragment"
                    - FRAGMENT_NAME@8..10
                        - NAME@8..10
                            - IDENT@8..10 "on"
                    - TYPE_CONDITION@10..14
                        - NAMED_TYPE@10..14
                            - NAME@10..14
                                - IDENT@10..14 "User"
                    - DIRECTIVES@14..22
                        - DIRECTIVE@14..22
                            - AT@14..15 "@"
                            - NAME@15..22
                                - IDENT@15..22 "example"
                    - SELECTION_SET@22..26
                        - L_CURLY@22..23 "{"
                        - SELECTION@23..25
                            - FIELD@23..25
                                - NAME@23..25
                                    - IDENT@23..25 "id"
                        - R_CURLY@25..26 "}"
            - ERROR@0:2 "Fragment Name cannot be 'on'"
            - ERROR@0:4 "Expected Fragment's Type Condition to have 'on', got User"
            "#,
        );
    }

    #[test]
    fn it_returns_error_with_invalid_type_condition() {
        utils::check_ast(
            "fragment friendFields User @example {
              id
            }
            ",
            r#"
            - DOCUMENT@0..36
                - FRAGMENT_DEFINITION@0..36
                    - fragment_KW@0..8 "fragment"
                    - FRAGMENT_NAME@8..20
                        - NAME@8..20
                            - IDENT@8..20 "friendFields"
                    - TYPE_CONDITION@20..24
                        - NAMED_TYPE@20..24
                            - NAME@20..24
                                - IDENT@20..24 "User"
                    - DIRECTIVES@24..32
                        - DIRECTIVE@24..32
                            - AT@24..25 "@"
                            - NAME@25..32
                                - IDENT@25..32 "example"
                    - SELECTION_SET@32..36
                        - L_CURLY@32..33 "{"
                        - SELECTION@33..35
                            - FIELD@33..35
                                - NAME@33..35
                                    - IDENT@33..35 "id"
                        - R_CURLY@35..36 "}"
            - ERROR@0:4 "Expected Fragment's Type Condition to have 'on', got User"
            "#,
        );
    }

    #[test]
    fn it_returns_error_with_invalid_selection_set() {
        utils::check_ast(
            "fragment friendFields on User",
            r#"
            - DOCUMENT@0..26
                - FRAGMENT_DEFINITION@0..26
                    - fragment_KW@0..8 "fragment"
                    - FRAGMENT_NAME@8..20
                        - NAME@8..20
                            - IDENT@8..20 "friendFields"
                    - TYPE_CONDITION@20..26
                        - on_KW@20..22 "on"
                        - NAMED_TYPE@22..26
                            - NAME@22..26
                                - IDENT@22..26 "User"
            - ERROR@0:15 "Expected Fragment Definition to have a Selection Set, got no further data"
            "#,
        );
    }
}
