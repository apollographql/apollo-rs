use crate::format_err;
use crate::{Parser, SyntaxKind, TokenKind};

/// See: https://spec.graphql.org/June2018/#Name
///
/// ```txt
/// Name
///     [_A-Za-z][_0-9A-Za-z]*/
/// ```
pub(crate) fn name(parser: &mut Parser) -> Result<(), crate::Error> {
    let _guard = parser.start_node(SyntaxKind::NAME);
    match parser.peek() {
        Some(TokenKind::Node) => {
            let data = parser.peek_data().unwrap();
            // TODO @lrlna: remove assert as this panics, and we want to collect errors
            assert!(data.starts_with(is_start_char));
            if data.len() >= 2 {
                assert!(data[1..].chars().all(is_remainder_char));
            }
            parser.bump(SyntaxKind::IDENT);
            Ok(())
        }
        // missing name
        _ => {
            return format_err!(
                parser.peek_data().unwrap(),
                "Expected a spec compliant Name, got {}",
                parser.peek_data().unwrap()
            )
        }
    }
}

fn is_start_char(c: char) -> bool {
    matches!(c, '_' | 'A'..='Z' | 'a'..='z')
}

fn is_remainder_char(c: char) -> bool {
    matches!(c, '_' | 'A'..='Z' | 'a'..='z' | '0'..='9')
}

pub(crate) fn named_type(parser: &mut Parser) -> Result<(), crate::Error> {
    let _guard = parser.start_node(SyntaxKind::NAMED_TYPE);
    name(parser)?;
    Ok(())
}
