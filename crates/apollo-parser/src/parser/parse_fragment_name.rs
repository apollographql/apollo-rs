use crate::{Parser, TokenKind};

/// See: https://spec.graphql.org/June2018/#FragmentName
///
/// ```txt
/// FragmentName
///     Name *but not* on
/// ```
pub(crate) fn parse_fragment_name(parser: &mut Parser) -> Result<(), ()> {
    match parser.peek() {
        Some(TokenKind::Node) => {
            if parser.peek_data().unwrap() == "on" {
                // fragment name cannot have "on" as part of its definition
                return Err(());
            }
            parser.bump();
            Ok(())
        }
        // missing fragment name
        _ => return Err(()),
    }
}
