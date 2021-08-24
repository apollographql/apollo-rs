use crate::parser::field;
use crate::{Parser, SyntaxKind, TokenKind};

/// See: https://spec.graphql.org/June2018/#SelectionSet
///
/// ```txt
/// SelectionSet
///     { Selection }
/// ```
pub(crate) fn selection_set(parser: &mut Parser) {
    if let Some(TokenKind::LCurly) = parser.peek() {
        let guard = parser.start_node(SyntaxKind::SELECTION_SET);
        parser.bump(SyntaxKind::L_CURLY);
        selection(parser);
        if let Some(TokenKind::RCurly) = parser.peek() {
            parser.bump(SyntaxKind::R_CURLY);
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
pub(crate) fn selection(parser: &mut Parser) {
    let _guard = parser.start_node(SyntaxKind::SELECTION);
    if let Some(TokenKind::Spread) = parser.peek() {
        if let Some(node) = parser.peek_data() {
            match node.as_str() {
                "on" => todo!(), // fragment::inline_fragment(parser)
                _ => todo!(),    // fragment::fragment_spread(parser)
            }
        }
    }
    while let Some(TokenKind::Node) = parser.peek() {
        field::field(parser)
    }
    // return create_err!(
    //     parser
    //         .peek_data()
    //         .unwrap_or_else(|| String::from("no further data")),
    //     "Selection can only be a Field, Fragment Spread or Inline Fragment, got {} ",
    //     parser
    //         .peek_data()
    //         .unwrap_or_else(|| String::from("no further data"))
    // );
}
