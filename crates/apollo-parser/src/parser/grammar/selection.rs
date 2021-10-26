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
