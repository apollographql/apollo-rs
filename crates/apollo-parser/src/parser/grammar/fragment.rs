use crate::parser::grammar::{directive, name, selection, ty};
use crate::{Parser, SyntaxKind, TokenKind, S, T};

/// See: https://spec.graphql.org/draft/#FragmentDefinition
///
/// *FragmentDefinition*:
///     **fragment** FragmentName TypeCondition Directives<sub>opt</sub> SelectionSet
pub(crate) fn fragment_definition(p: &mut Parser) {
    let _g = p.start_node(SyntaxKind::FRAGMENT_DEFINITION);
    p.bump(SyntaxKind::fragment_KW);

    fragment_name(p);
    type_condition(p);

    if let Some(T![@]) = p.peek() {
        directive::directives(p);
    }

    match p.peek() {
        Some(T!['{']) => selection::selection_set(p),
        _ => p.err("expected a Selection Set"),
    }
}

/// See: https://spec.graphql.org/draft/#FragmentName
///
/// *FragmentName*:
///     Name *but not* **on**
pub(crate) fn fragment_name(p: &mut Parser) {
    let _g = p.start_node(SyntaxKind::FRAGMENT_NAME);
    match p.peek() {
        Some(TokenKind::Name) => {
            if p.peek_data().unwrap() == "on" {
                return p.err("Fragment Name cannot be 'on'");
            }
            name::name(p)
        }
        _ => p.err("expected Fragment Name"),
    }
}

/// See: https://spec.graphql.org/draft/#TypeCondition
///
/// *TypeCondition*:
///     **on** NamedType
pub(crate) fn type_condition(p: &mut Parser) {
    let _g = p.start_node(SyntaxKind::TYPE_CONDITION);
    match p.peek() {
        Some(TokenKind::Name) => {
            if p.peek_data().unwrap() == "on" {
                p.bump(SyntaxKind::on_KW);
            } else {
                p.err("exptected 'on'");
            }
            ty::named_type(p)
        }
        _ => p.err("expected Type Condition"),
    }
}

/// See: https://spec.graphql.org/draft/#InlineFragment
///
/// *InlineFragment*:
///     **...** TypeCondition<sub>opt</sub> Directives<sub>opt</sub> SelectionSet
pub(crate) fn inline_fragment(p: &mut Parser) {
    let _g = p.start_node(SyntaxKind::INLINE_FRAGMENT);
    p.bump(S![...]);

    if let Some(TokenKind::Name) = p.peek() {
        type_condition(p);
    }

    if let Some(T![@]) = p.peek() {
        directive::directives(p);
    }

    match p.peek() {
        Some(T!['{']) => selection::selection_set(p),
        _ => p.err("expected Selection Set"),
    }
}

/// See: https://spec.graphql.org/draft/#FragmentSpread
///
/// *FragmentSpread*:
///     **...** FragmentName Directives<sub>opt</sub>
pub(crate) fn fragment_spread(p: &mut Parser) {
    let _g = p.start_node(SyntaxKind::FRAGMENT_SPREAD);
    p.bump(S![...]);

    match p.peek() {
        Some(TokenKind::Name) => {
            fragment_name(p);
        }
        _ => p.err("expected a Name"),
    }

    if let Some(T![@]) = p.peek() {
        directive::directives(p);
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
            - DOCUMENT@0..138
                - FRAGMENT_DEFINITION@0..138
                    - fragment_KW@0..8 "fragment"
                    - WHITESPACE@8..9 " "
                    - FRAGMENT_NAME@9..22
                        - NAME@9..22
                            - IDENT@9..21 "friendFields"
                            - WHITESPACE@21..22 " "
                    - TYPE_CONDITION@22..30
                        - on_KW@22..24 "on"
                        - WHITESPACE@24..25 " "
                        - NAMED_TYPE@25..30
                            - NAME@25..30
                                - IDENT@25..29 "User"
                                - WHITESPACE@29..30 " "
                    - DIRECTIVES@30..39
                        - DIRECTIVE@30..39
                            - AT@30..31 "@"
                            - NAME@31..39
                                - IDENT@31..38 "example"
                                - WHITESPACE@38..39 " "
                    - SELECTION_SET@39..125
                        - L_CURLY@39..40 "{"
                        - WHITESPACE@40..55 "\n              "
                        - SELECTION@55..124
                            - FIELD@55..72
                                - NAME@55..72
                                    - IDENT@55..57 "id"
                                    - WHITESPACE@57..72 "\n              "
                            - FIELD@72..91
                                - NAME@72..91
                                    - IDENT@72..76 "name"
                                    - WHITESPACE@76..91 "\n              "
                            - FIELD@91..124
                                - NAME@91..101
                                    - IDENT@91..101 "profilePic"
                                - ARGUMENTS@101..111
                                    - L_PAREN@101..102 "("
                                    - ARGUMENT@102..110
                                        - NAME@102..106
                                            - IDENT@102..106 "size"
                                        - COLON@106..107 ":"
                                        - WHITESPACE@107..108 " "
                                        - VALUE@108..110
                                            - INT_VALUE@108..110 "50"
                                    - R_PAREN@110..111 ")"
                                - WHITESPACE@111..124 "\n            "
                        - R_CURLY@124..125 "}"
                    - WHITESPACE@125..138 "\n            "
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
            - DOCUMENT@0..123
                - FRAGMENT_DEFINITION@0..123
                    - fragment_KW@0..8 "fragment"
                    - WHITESPACE@8..9 " "
                    - FRAGMENT_NAME@9..22
                        - NAME@9..22
                            - IDENT@9..21 "friendFields"
                            - WHITESPACE@21..22 " "
                    - TYPE_CONDITION@22..30
                        - on_KW@22..24 "on"
                        - WHITESPACE@24..25 " "
                        - NAMED_TYPE@25..30
                            - NAME@25..30
                                - IDENT@25..29 "User"
                                - WHITESPACE@29..30 " "
                    - SELECTION_SET@30..123
                        - L_CURLY@30..31 "{"
                        - WHITESPACE@31..48 "\n                "
                        - SELECTION@48..122
                            - FIELD@48..67
                                - NAME@48..67
                                    - IDENT@48..50 "id"
                                    - WHITESPACE@50..67 "\n                "
                            - FIELD@67..88
                                - NAME@67..88
                                    - IDENT@67..71 "name"
                                    - WHITESPACE@71..88 "\n                "
                            - FRAGMENT_SPREAD@88..122
                                - SPREAD@88..91 "..."
                                - FRAGMENT_NAME@91..122
                                    - NAME@91..122
                                        - IDENT@91..109 "standardProfilePic"
                                        - WHITESPACE@109..122 "\n            "
                        - R_CURLY@122..123 "}"
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
            - DOCUMENT@0..71
                - FRAGMENT_DEFINITION@0..71
                    - fragment_KW@0..8 "fragment"
                    - WHITESPACE@8..9 " "
                    - FRAGMENT_NAME@9..9
                    - TYPE_CONDITION@9..17
                        - on_KW@9..11 "on"
                        - WHITESPACE@11..12 " "
                        - NAMED_TYPE@12..17
                            - NAME@12..17
                                - IDENT@12..16 "User"
                                - WHITESPACE@16..17 " "
                    - DIRECTIVES@17..26
                        - DIRECTIVE@17..26
                            - AT@17..18 "@"
                            - NAME@18..26
                                - IDENT@18..25 "example"
                                - WHITESPACE@25..26 " "
                    - SELECTION_SET@26..58
                        - L_CURLY@26..27 "{"
                        - WHITESPACE@27..42 "\n              "
                        - SELECTION@42..57
                            - FIELD@42..57
                                - NAME@42..57
                                    - IDENT@42..44 "id"
                                    - WHITESPACE@44..57 "\n            "
                        - R_CURLY@57..58 "}"
                    - WHITESPACE@58..71 "\n            "
            - ERROR@0:2 "Fragment Name cannot be 'on'"
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
            - DOCUMENT@0..81
                - FRAGMENT_DEFINITION@0..81
                    - fragment_KW@0..8 "fragment"
                    - WHITESPACE@8..9 " "
                    - FRAGMENT_NAME@9..22
                        - NAME@9..22
                            - IDENT@9..21 "friendFields"
                            - WHITESPACE@21..22 " "
                    - TYPE_CONDITION@22..27
                        - NAMED_TYPE@22..27
                            - NAME@22..27
                                - IDENT@22..26 "User"
                                - WHITESPACE@26..27 " "
                    - DIRECTIVES@27..36
                        - DIRECTIVE@27..36
                            - AT@27..28 "@"
                            - NAME@28..36
                                - IDENT@28..35 "example"
                                - WHITESPACE@35..36 " "
                    - SELECTION_SET@36..68
                        - L_CURLY@36..37 "{"
                        - WHITESPACE@37..52 "\n              "
                        - SELECTION@52..67
                            - FIELD@52..67
                                - NAME@52..67
                                    - IDENT@52..54 "id"
                                    - WHITESPACE@54..67 "\n            "
                        - R_CURLY@67..68 "}"
                    - WHITESPACE@68..81 "\n            "
            - ERROR@0:4 "exptected 'on'"
            "#,
        );
    }

    #[test]
    fn it_returns_error_with_invalid_selection_set() {
        utils::check_ast(
            "fragment friendFields on User",
            r#"
            - DOCUMENT@0..29
                - FRAGMENT_DEFINITION@0..29
                    - fragment_KW@0..8 "fragment"
                    - WHITESPACE@8..9 " "
                    - FRAGMENT_NAME@9..22
                        - NAME@9..22
                            - IDENT@9..21 "friendFields"
                            - WHITESPACE@21..22 " "
                    - TYPE_CONDITION@22..29
                        - on_KW@22..24 "on"
                        - WHITESPACE@24..25 " "
                        - NAMED_TYPE@25..29
                            - NAME@25..29
                                - IDENT@25..29 "User"
            - ERROR@0:3 "expected a Selection Set"
            "#,
        );
    }
}
