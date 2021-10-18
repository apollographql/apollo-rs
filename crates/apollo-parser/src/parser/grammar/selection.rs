use crate::{
    parser::grammar::{field, fragment},
    Parser, SyntaxKind, TokenKind, S, T,
};

/// See: https://spec.graphql.org/draft/#SelectionSet
///
/// *SelectionSet*:
///     **{** Selection<sub>list</sub> **}**
pub(crate) fn selection_set(p: &mut Parser) {
    if let Some(T!['{']) = p.peek() {
        let _g = p.start_node(SyntaxKind::SELECTION_SET);
        p.bump(S!['{']);
        selection(p);
        p.expect(T!['}'], S!['}']);
    }
}

/// See: https://spec.graphql.org/draft/#Selection
///
/// *Selection*:
///     Field
///     FragmentSpread
///     InlineFragment
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
            - DOCUMENT@0..189
                - OPERATION_DEFINITION@0..189
                    - SELECTION_SET@0..189
                        - L_CURLY@0..1 "{"
                        - WHITESPACE@1..18 "\n                "
                        - SELECTION@18..188
                            - FIELD@18..46
                                - ALIAS@18..26
                                    - NAME@18..24
                                        - IDENT@18..24 "animal"
                                    - COLON@24..25 ":"
                                    - WHITESPACE@25..26 " "
                                - NAME@26..46
                                    - IDENT@26..29 "cat"
                                    - WHITESPACE@29..46 "\n                "
                            - FIELD@46..171
                                - NAME@46..50
                                    - IDENT@46..49 "dog"
                                    - WHITESPACE@49..50 " "
                                - SELECTION_SET@50..171
                                    - L_CURLY@50..51 "{"
                                    - WHITESPACE@51..72 "\n                    "
                                    - SELECTION@72..153
                                        - FIELD@72..153
                                            - NAME@72..78
                                                - IDENT@72..77 "panda"
                                                - WHITESPACE@77..78 " "
                                            - SELECTION_SET@78..153
                                                - L_CURLY@78..79 "{"
                                                - WHITESPACE@79..104 "\n                        "
                                                - SELECTION@104..135
                                                    - FIELD@104..135
                                                        - NAME@104..135
                                                            - IDENT@104..114 "anotherCat"
                                                            - WHITESPACE@114..135 "\n                    "
                                                - R_CURLY@135..136 "}"
                                                - WHITESPACE@136..153 "\n                "
                                    - R_CURLY@153..154 "}"
                                    - WHITESPACE@154..171 "\n                "
                            - FIELD@171..188
                                - NAME@171..188
                                    - IDENT@171..175 "lion"
                                    - WHITESPACE@175..188 "\n            "
                        - R_CURLY@188..189 "}"
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
            - DOCUMENT@0..144
                - OPERATION_DEFINITION@0..144
                    - SELECTION_SET@0..144
                        - L_CURLY@0..1 "{"
                        - WHITESPACE@1..18 "\n                "
                        - SELECTION@18..143
                            - FIELD@18..38
                                - NAME@18..38
                                    - IDENT@18..21 "cat"
                                    - WHITESPACE@21..38 "\n                "
                            - FIELD@38..58
                                - NAME@38..58
                                    - IDENT@38..41 "dog"
                                    - WHITESPACE@41..58 "\n                "
                            - INLINE_FRAGMENT@58..143
                                - SPREAD@58..61 "..."
                                - WHITESPACE@61..62 " "
                                - TYPE_CONDITION@62..72
                                    - on_KW@62..64 "on"
                                    - WHITESPACE@64..65 " "
                                    - NAMED_TYPE@65..72
                                        - NAME@65..72
                                            - IDENT@65..71 "Animal"
                                            - WHITESPACE@71..72 " "
                                - DIRECTIVES@72..81
                                    - DIRECTIVE@72..81
                                        - AT@72..73 "@"
                                        - NAME@73..81
                                            - IDENT@73..80 "example"
                                            - WHITESPACE@80..81 " "
                                - SELECTION_SET@81..143
                                    - L_CURLY@81..82 "{"
                                    - WHITESPACE@82..103 "\n                    "
                                    - SELECTION@103..129
                                        - FIELD@103..129
                                            - NAME@103..129
                                                - IDENT@103..112 "treatKind"
                                                - WHITESPACE@112..129 "\n                "
                                    - R_CURLY@129..130 "}"
                                    - WHITESPACE@130..143 "\n            "
                        - R_CURLY@143..144 "}"
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
            - DOCUMENT@0..91
                - OPERATION_DEFINITION@0..91
                    - SELECTION_SET@0..91
                        - L_CURLY@0..1 "{"
                        - WHITESPACE@1..18 "\n                "
                        - SELECTION@18..90
                            - FIELD@18..38
                                - NAME@18..38
                                    - IDENT@18..21 "pet"
                                    - WHITESPACE@21..38 "\n                "
                            - FIELD@38..60
                                - NAME@38..60
                                    - IDENT@38..43 "treat"
                                    - WHITESPACE@43..60 "\n                "
                            - FRAGMENT_SPREAD@60..90
                                - SPREAD@60..63 "..."
                                - FRAGMENT_NAME@63..90
                                    - NAME@63..90
                                        - IDENT@63..77 "snackSelection"
                                        - WHITESPACE@77..90 "\n            "
                        - R_CURLY@90..91 "}"
            "#,
        )
    }
}
