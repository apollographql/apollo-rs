use crate::parser::field;
use crate::{format_err, Parser, SyntaxKind, TokenKind};

/// See: https://spec.graphql.org/June2018/#SelectionSet
///
/// ```txt
/// SelectionSet
///     { Selection }
/// ```
pub(crate) fn selection_set(parser: &mut Parser) -> Result<(), crate::Error> {
    let _guard = parser.start_node(SyntaxKind::SELECTION_SET);
    parser.bump(SyntaxKind::L_CURLY);
    selection(parser)?;
    Ok(())
}

/// See: https://spec.graphql.org/June2018/#Selection
///
/// ```txt
/// Selection
///     Field
///     FragmentSpread
///     InlineFragment
/// ```
pub(crate) fn selection(parser: &mut Parser) -> Result<(), crate::Error> {
    let _guard = parser.start_node(SyntaxKind::SELECTION);
    match parser.peek() {
        Some(TokenKind::Spread) => {
            if let Some(TokenKind::On) = parser.peek() {
                todo!();
                // fragment::inline_fragment(parser)?;
            } else {
                todo!();
                // framgent::fragment_spread(parser)?;
            }
        }
        Some(TokenKind::Node) => field::field(parser)?,
        _ => {
            return format_err!(
                parser
                    .peek_data()
                    .unwrap_or_else(|| String::from("no further data")),
                "Selection can only be a Field, Fragment Spread or Inline Fragment, got {} ",
                parser
                    .peek_data()
                    .unwrap_or_else(|| String::from("no further data"))
            )
        }
    }

    Ok(())
}
