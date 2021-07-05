use crate::{Parser, TokenKind};

use super::parse_fragment_name;

/// See: https://spec.graphql.org/June2018/#sec-Language.Fragments
///
/// ```txt
/// FragmentDefinition
///     fragment FragmentName TypeCondition Directives(opt) SelectionSet
/// ```
pub(crate) fn parse_fragment(parser: &mut Parser) -> Result<(), ()> {
    parser.builder.start_node(TokenKind::Fragment.into());
    parser.bump();
    // parser.parse_whitespace();
    parse_fragment_name(parser)?;

    // TODO(lrlna): parse TypeCondition, Directives, SelectionSet
    parser.builder.finish_node();
    Ok(())
}
