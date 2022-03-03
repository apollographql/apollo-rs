use crate::{Parser, SyntaxKind};

/// See: https://spec.graphql.org/October2021/#Description
///
/// *Description*:
///     StringValue
pub(crate) fn description(p: &mut Parser) {
    let _g = p.start_node(SyntaxKind::DESCRIPTION);
    let _g_string = p.start_node(SyntaxKind::STRING_VALUE);
    p.bump(SyntaxKind::STRING)
}
