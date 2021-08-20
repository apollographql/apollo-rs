use crate::{create_err, parser::name, Parser, SyntaxKind, TokenKind};

/// See: https://spec.graphql.org/June2018/#Argument
///
/// ```txt
/// Argument
///    Name : Value
/// ```
pub(crate) fn argument(parser: &mut Parser, is_argument: bool) {
    if let Some(TokenKind::Node) = parser.peek() {
        let guard = parser.start_node(SyntaxKind::ARGUMENT);
        name::name(parser);
        if let Some(TokenKind::Colon) = parser.peek() {
            parser.bump(SyntaxKind::COLON);
            if let Some(TokenKind::Node) = parser.peek() {
                // TODO @lrlna: use value type function
                parser.bump(SyntaxKind::VALUE);
                if parser.peek().is_some() {
                    guard.finish_node();
                    return argument(parser, true);
                }
            }
            parser.push_err(create_err!(
                parser
                    .peek_data()
                    .unwrap_or_else(|| String::from("no further data")),
                "Expected Argument to have a Value, got {}",
                parser
                    .peek_data()
                    .unwrap_or_else(|| String::from("no further data"))
            ));
        }
    } else {
        parser.push_err(create_err!(
            parser
                .peek_data()
                .unwrap_or_else(|| String::from("no further data")),
            "Expected Argument to have a Name, got {}",
            parser
                .peek_data()
                .unwrap_or_else(|| String::from("no further data"))
        ));
    }
    if let Some(TokenKind::Comma) = parser.peek() {
        parser.bump(SyntaxKind::COMMA);
        return argument(parser, is_argument);
    }
    if !is_argument {
        parser.push_err(create_err!(
            parser
                .peek_data()
                .unwrap_or_else(|| String::from("no further data")),
            "Expected to have an Argument, got {}",
            parser
                .peek_data()
                .unwrap_or_else(|| String::from("no further data"))
        ));
    }
}

/// See: https://spec.graphql.org/June2018/#Arguments
///
/// ```txt
/// Arguments
///    ( Argument(list) )
/// ```
pub(crate) fn arguments(parser: &mut Parser) {
    let guard = parser.start_node(SyntaxKind::ARGUMENTS);
    parser.bump(SyntaxKind::L_PAREN);
    argument(parser, false);
    match parser.peek() {
        Some(TokenKind::RParen) => {
            parser.bump(SyntaxKind::R_PAREN);
            guard.finish_node();
        }
        // missing a closing RParen
        _ => parser.push_err(create_err!(
            parser
                .peek_data()
                .unwrap_or_else(|| String::from("no further data")),
            "Expected closing ')', got {}",
            parser
                .peek_data()
                .unwrap_or_else(|| String::from("no further data"))
        )),
    }
}
