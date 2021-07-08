use crate::{Parser, SyntaxKind};

use super::parse_fragment_name;

/// See: https://spec.graphql.org/June2018/#sec-Language.Fragments
///
/// ```txt
/// FragmentDefinition
///     fragment FragmentName TypeCondition Directives(opt) SelectionSet
/// ```
pub(crate) fn parse_fragment(parser: &mut Parser) -> Result<(), ()> {
    let _guard = parser.start_node(SyntaxKind::FRAGMENT_DEFINITION);
    // TODO lrlna: parse description???
    parser.bump(SyntaxKind::fragment_KW);
    // parser.parse_whitespace();
    parse_fragment_name(parser)?;

    // TODO(lrlna): parse TypeCondition, Directives, SelectionSet

    Ok(())
}
