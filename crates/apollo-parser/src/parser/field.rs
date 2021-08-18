use crate::parser::{argument, directive, name, selection};
use crate::{format_err, Parser, SyntaxKind, TokenKind};

/// See: https://spec.graphql.org/June2018/#Field
///
/// ```txt
/// Field
///     Alias(opt) Name Arguments(opt) Directives(opt) SelectionSet(opt)
/// ```
pub(crate) fn field(parser: &mut Parser) -> Result<(), crate::Error> {
    let guard = parser.start_node(SyntaxKind::FIELD);
    if let Some(TokenKind::Node) = parser.peek() {
        if let Some(TokenKind::Colon) = parser.peek_n(2) {
            name::alias(parser)?
        }
        name::name(parser)?
    } else {
        return format_err!(
            parser
                .peek_data()
                .unwrap_or_else(|| String::from("no further data")),
            "Expected Field to have a Name, got {}",
            parser
                .peek_data()
                .unwrap_or_else(|| String::from("no further data"))
        );
    }
    match parser.peek() {
        // arguments
        Some(TokenKind::LParen) => argument::arguments(parser)?,
        // directives
        Some(TokenKind::At) => directive::directives(parser)?,
        // Selection Set
        Some(TokenKind::LCurly) => selection::selection_set(parser)?,
        // Selection Set
        Some(TokenKind::Comma) => {
            guard.finish_node();
            parser.bump(SyntaxKind::COMMA);
            field(parser)?
        }
        // Another Field
        Some(TokenKind::Node) => {
            guard.finish_node();
            field(parser)?
        }
        Some(TokenKind::RCurly) => {
            guard.finish_node();
        }
        _ => guard.finish_node(),
    }
    Ok(())
}
