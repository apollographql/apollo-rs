use crate::{Parser, SyntaxKind, TokenKind, S};

/// See: https://spec.graphql.org/October2021/#Name
///
/// *Name*:
///     [_A-Za-z][_0-9A-Za-z]
pub(crate) fn name(p: &mut Parser) {
    let _g = p.start_node(SyntaxKind::NAME);
    match p.peek() {
        Some(TokenKind::Name) => {
            validate_name(&p.peek_data().unwrap(), p);
            p.bump(SyntaxKind::IDENT);
        }
        _ => p.err("expected a Name"),
    }
}

pub(crate) fn validate_name(name: &str, p: &mut Parser) {
    if !name.starts_with(is_start_char) {
        p.err_and_pop("expected Name to start with a letter or an _");
    }
    if name.len() >= 2 && !name[1..].chars().all(is_remainder_char) {
        p.err_and_pop("Name can only be composed of letters, numbers and _");
    }
}

/// See: https://spec.graphql.org/October2021/#Alias
///
/// *Alias*:
///     Name **:**
pub(crate) fn alias(p: &mut Parser) {
    let _g = p.start_node(SyntaxKind::ALIAS);
    name(p);
    p.bump(S![:]);
}

fn is_start_char(c: char) -> bool {
    matches!(c, '_' | 'A'..='Z' | 'a'..='z')
}

fn is_remainder_char(c: char) -> bool {
    matches!(c, '_' | 'A'..='Z' | 'a'..='z' | '0'..='9')
}
