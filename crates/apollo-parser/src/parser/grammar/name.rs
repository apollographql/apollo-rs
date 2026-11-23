use crate::Parser;
use crate::SyntaxKind;
use crate::TokenKind;
use crate::S;

/// See: https://spec.graphql.org/September2025/#Name
///
/// *Name*:
///     [_A-Za-z][_0-9A-Za-z]
pub(crate) fn name(p: &mut Parser) {
    match p.peek_token() {
        Some(token) if token.kind() == TokenKind::Name => {
            let _g = p.start_node(SyntaxKind::NAME);
            p.bump(SyntaxKind::IDENT);
        }
        _ => p.err("expected a Name"),
    }
}

/// See: https://spec.graphql.org/September2025/#Alias
///
/// *Alias*:
///     Name **:**
pub(crate) fn alias(p: &mut Parser) {
    let _g = p.start_node(SyntaxKind::ALIAS);
    name(p);
    p.bump(S![:]);
}
