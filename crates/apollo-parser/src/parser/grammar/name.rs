use crate::create_err;
use crate::{Parser, SyntaxKind, TokenKind};

/// See: https://spec.graphql.org/June2018/#Name
///
/// ```txt
/// Name
///     [_A-Za-z][_0-9A-Za-z]*/
/// ```
pub(crate) fn name(parser: &mut Parser) {
    let _guard = parser.start_node(SyntaxKind::NAME);
    match parser.peek() {
        Some(TokenKind::Node) => {
            validate_name(parser);
            parser.bump(SyntaxKind::IDENT);
        }
        // missing name
        _ => parser.push_err(create_err!(
            parser.peek_data().unwrap(),
            "Expected a spec compliant Name, got {}",
            parser.peek_data().unwrap()
        )),
    }
}

pub(crate) fn validate_name(parser: &mut Parser) {
    let data = parser.peek_data().unwrap();
    if !data.starts_with(is_start_char) {
        parser.push_err(create_err!(
            parser.peek_data().unwrap(),
            "Expected Name to start with a letter or an _, got {}",
            parser.peek_data().unwrap()
        ));
    }
    if data.len() >= 2 && !data[1..].chars().all(is_remainder_char) {
        parser.push_err(create_err!(
            parser.peek_data().unwrap(),
            "Name can only be composed of letters, numbers and _, got {}",
            parser.peek_data().unwrap()
        ));
    }
}

/// See: https://spec.graphql.org/June2018/#Alias
///
/// ```txt
/// Alias
///     Name :
/// ```
pub(crate) fn alias(parser: &mut Parser) {
    let _guard = parser.start_node(SyntaxKind::ALIAS);
    name(parser);
    parser.bump(SyntaxKind::COLON);
}

fn is_start_char(c: char) -> bool {
    matches!(c, '_' | 'A'..='Z' | 'a'..='z')
}

fn is_remainder_char(c: char) -> bool {
    matches!(c, '_' | 'A'..='Z' | 'a'..='z' | '0'..='9')
}
