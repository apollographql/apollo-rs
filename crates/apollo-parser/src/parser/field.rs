use crate::parser::name;
use crate::{format_err, Parser, SyntaxKind, TokenKind};

/// See: https://spec.graphql.org/June2018/#Field
///
/// ```txt
/// Field
///     Alias(opt) Name Arguments(opt) Directives(opt) SelectionSet(opt)
/// ```
pub(crate) fn field(parser: &mut Parser) -> Result<(), crate::Error> {
    let _guard = parser.start_node(SyntaxKind::FIELD);
    match parser.peek() {
        Some(TokenKind::Node) => match parser.peek() {
            Some(TokenKind::Colon) => name::alias(parser)?,
            _ => name::name(parser)?,
        },
        _ => {
            return format_err!(
                parser
                    .peek_data()
                    .unwrap_or_else(|| String::from("no further data")),
                "Expected closing ')', got {}",
                parser
                    .peek_data()
                    .unwrap_or_else(|| String::from("no further data"))
            );
        }
    }
    Ok(())
}
