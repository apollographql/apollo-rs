use crate::parser::grammar::{field, fragment};
use crate::{Parser, SyntaxKind, TokenKind, S, T};

/// See: https://spec.graphql.org/June2018/#SelectionSet
///
/// ```txt
/// SelectionSet
///     { Selection }
/// ```
pub(crate) fn selection_set(p: &mut Parser) {
    if let Some(T!['{']) = p.peek() {
        let guard = p.start_node(SyntaxKind::SELECTION_SET);
        p.bump(S!['{']);
        selection(p);
        if let Some(T!['}']) = p.peek() {
            p.bump(S!['}']);
            guard.finish_node()
        }
    }
}

/// See: https://spec.graphql.org/June2018/#Selection
///
/// ```txt
/// Selection
///     Field
///     FragmentSpread
///     InlineFragment
/// ```
pub(crate) fn selection(p: &mut Parser) {
    let guard = p.start_node(SyntaxKind::SELECTION);
    while let Some(node) = p.peek() {
        match node {
            T![...] => {
                if let Some(node) = p.peek_data_n(2) {
                    match node.as_str() {
                        "on" | "{" => fragment::inline_fragment(p),
                        _ => fragment::fragment_spread(p),
                    }
                } else {
                    p.err("expected an Inline Fragment or a Fragment Spread");
                }
            }
            T!['{'] => {
                guard.finish_node();
                break;
            }
            TokenKind::Name => {
                field::field(p);
            }
            _ => break,
        }
    }
}

#[cfg(test)]
mod test {
    use crate::parser::utils;

    #[test]
    fn it_parses_selection_set() {
        utils::check_ast(
            "{
                animal: cat
                dog {
                    panda {
                        anotherCat
                    }
                }
                lion
            }",
            r#"
            - DOCUMENT@0..38
                - OPERATION_DEFINITION@0..38
                    - SELECTION_SET@0..38
                        - L_CURLY@0..1 "{"
                        - SELECTION@1..37
                            - FIELD@1..11
                                - ALIAS@1..8
                                    - NAME@1..7
                                        - IDENT@1..7 "animal"
                                    - COLON@7..8 ":"
                                - NAME@8..11
                                    - IDENT@8..11 "cat"
                            - FIELD@11..33
                                - NAME@11..14
                                    - IDENT@11..14 "dog"
                                - SELECTION_SET@14..33
                                    - L_CURLY@14..15 "{"
                                    - SELECTION@15..32
                                        - FIELD@15..32
                                            - NAME@15..20
                                                - IDENT@15..20 "panda"
                                            - SELECTION_SET@20..32
                                                - L_CURLY@20..21 "{"
                                                - SELECTION@21..31
                                                    - FIELD@21..31
                                                        - NAME@21..31
                                                            - IDENT@21..31 "anotherCat"
                                                - R_CURLY@31..32 "}"
                                    - R_CURLY@32..33 "}"
                            - FIELD@33..37
                                - NAME@33..37
                                    - IDENT@33..37 "lion"
                        - R_CURLY@37..38 "}"
            "#,
        )
    }

    #[test]
    fn it_parses_selection_with_inline_fragment() {
        utils::check_ast(
            "{
                cat
                dog
                ... on Animal @example {
                    treatKind
                }
            }",
            r#"
            - DOCUMENT@0..38
                - OPERATION_DEFINITION@0..38
                    - SELECTION_SET@0..38
                        - L_CURLY@0..1 "{"
                        - SELECTION@1..37
                            - FIELD@1..4
                                - NAME@1..4
                                    - IDENT@1..4 "cat"
                            - FIELD@4..7
                                - NAME@4..7
                                    - IDENT@4..7 "dog"
                            - INLINE_FRAGMENT@7..37
                                - SPREAD@7..10 "..."
                                - TYPE_CONDITION@10..18
                                    - on_KW@10..12 "on"
                                    - NAMED_TYPE@12..18
                                        - NAME@12..18
                                            - IDENT@12..18 "Animal"
                                - DIRECTIVES@18..26
                                    - DIRECTIVE@18..26
                                        - AT@18..19 "@"
                                        - NAME@19..26
                                            - IDENT@19..26 "example"
                                - SELECTION_SET@26..37
                                    - L_CURLY@26..27 "{"
                                    - SELECTION@27..36
                                        - FIELD@27..36
                                            - NAME@27..36
                                                - IDENT@27..36 "treatKind"
                                    - R_CURLY@36..37 "}"
                        - R_CURLY@37..38 "}"
            "#,
        )
    }

    #[test]
    fn it_parses_selection_with_fragment_spread() {
        utils::check_ast(
            "{
                pet
                treat
                ...snackSelection
            }",
            r#"
            - DOCUMENT@0..27
                - OPERATION_DEFINITION@0..27
                    - SELECTION_SET@0..27
                        - L_CURLY@0..1 "{"
                        - SELECTION@1..26
                            - FIELD@1..4
                                - NAME@1..4
                                    - IDENT@1..4 "pet"
                            - FIELD@4..9
                                - NAME@4..9
                                    - IDENT@4..9 "treat"
                            - FRAGMENT_SPREAD@9..26
                                - SPREAD@9..12 "..."
                                - FRAGMENT_NAME@12..26
                                    - NAME@12..26
                                        - IDENT@12..26 "snackSelection"
                        - R_CURLY@26..27 "}"
            "#,
        )
    }
}
