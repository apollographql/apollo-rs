use crate::create_err;
use crate::{Parser, SyntaxKind, TokenKind, S};

/// See: https://spec.graphql.org/June2018/#Name
///
/// ```txt
/// Name
///     [_A-Za-z][_0-9A-Za-z]*/
/// ```
pub(crate) fn name(p: &mut Parser) {
    let _guard = p.start_node(SyntaxKind::NAME);
    match p.peek() {
        Some(TokenKind::Name) => {
            validate_name(p);
            p.bump(SyntaxKind::IDENT);
        }
        // missing name
        _ => p.push_err(create_err!(
            p.peek_data().unwrap(),
            "Expected a spec compliant Name, got {}",
            p.peek_data().unwrap()
        )),
    }
}

pub(crate) fn validate_name(p: &mut Parser) {
    let data = p.peek_data().unwrap();
    if !data.starts_with(is_start_char) {
        p.push_err(create_err!(
            p.peek_data().unwrap(),
            "Expected Name to start with a letter or an _, got {}",
            p.peek_data().unwrap()
        ));
    }
    if data.len() >= 2 && !data[1..].chars().all(is_remainder_char) {
        p.push_err(create_err!(
            p.peek_data().unwrap(),
            "Name can only be composed of letters, numbers and _, got {}",
            p.peek_data().unwrap()
        ));
    }
}

/// See: https://spec.graphql.org/June2018/#Alias
///
/// ```txt
/// Alias
///     Name :
/// ```
pub(crate) fn alias(p: &mut Parser) {
    let _guard = p.start_node(SyntaxKind::ALIAS);
    name(p);
    p.bump(S![:]);
}

fn is_start_char(c: char) -> bool {
    matches!(c, '_' | 'A'..='Z' | 'a'..='z')
}

fn is_remainder_char(c: char) -> bool {
    matches!(c, '_' | 'A'..='Z' | 'a'..='z' | '0'..='9')
}
