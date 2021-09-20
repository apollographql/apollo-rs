use crate::parser::grammar::{directive, name, selection, ty};
use crate::{Parser, SyntaxKind, TokenKind, S, T};

/// See: https://spec.graphql.org/June2018/#FragmentDefinition
///
/// ```txt
/// FragmentDefinition
///     fragment FragmentName TypeCondition Directives(opt) SelectionSet
/// ```
pub(crate) fn fragment_definition(p: &mut Parser) {
    let _guard = p.start_node(SyntaxKind::FRAGMENT_DEFINITION);
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

/// See: https://spec.graphql.org/June2018/#FragmentName
///
/// ```txt
/// FragmentName
///     Name *but not* on
/// ```
pub(crate) fn fragment_name(p: &mut Parser) {
    let _guard = p.start_node(SyntaxKind::FRAGMENT_NAME);
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

/// See: https://spec.graphql.org/June2018/#TypeCondition
///
/// ```txt
/// TypeCondition
///     on NamedType
/// ```
pub(crate) fn type_condition(p: &mut Parser) {
    let _guard = p.start_node(SyntaxKind::TYPE_CONDITION);
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

/// See: https://spec.graphql.org/June2018/#InlineFragment
///
/// ```txt
/// InlineFragment
///     ... TypeCondition[opt] Directives[opt] SelectionSet
/// ```
pub(crate) fn inline_fragment(p: &mut Parser) {
    let _guard = p.start_node(SyntaxKind::INLINE_FRAGMENT);
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

/// See: https://spec.graphql.org/June2018/#FragmentSpread
///
/// ```txt
/// FragmentSpread
///     ... FragmentName Directives[opt]
/// ```
pub(crate) fn fragment_spread(p: &mut Parser) {
    let _guard = p.start_node(SyntaxKind::FRAGMENT_SPREAD);
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
                    - FRAGMENT_NAME@8..8
                    - TYPE_CONDITION@8..14
                        - on_KW@8..10 "on"
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
            - ERROR@0:4 "exptected 'on'"
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
            - ERROR@0:3 "expected a Selection Set"
            "#,
        );
    }
}
