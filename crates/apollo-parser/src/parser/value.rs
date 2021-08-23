use crate::parser::{name, variable};
use crate::{create_err, Parser, SyntaxKind, TokenKind};

/// See: https://spec.graphql.org/June2018/#Value
///
/// ```txt
/// Value [Const]
///     [~Const] Variable
///     IntValue
///     FloatValue
///     StringValue
///     BooleanValue
///     NullValue
///     EnumValue
///     ListValue [Const]
///     ObjectValue [Const]
/// ```
pub(crate) fn value(parser: &mut Parser) {
    let _guard = parser.start_node(SyntaxKind::VALUE);
    match parser.peek() {
        Some(TokenKind::Dollar) => variable::variable(parser),
        Some(TokenKind::Int) => parser.bump(SyntaxKind::INT_VALUE),
        Some(TokenKind::Float) => parser.bump(SyntaxKind::FLOAT_VALUE),
        Some(TokenKind::StringValue) => parser.bump(SyntaxKind::STRING_VALUE),
        Some(TokenKind::Boolean) => parser.bump(SyntaxKind::BOOLEAN_VALUE),
        Some(TokenKind::Null) => parser.bump(SyntaxKind::NULL_VALUE),
        Some(TokenKind::Node) => name::name(parser),
        Some(TokenKind::LBracket) => list_value(parser),
        Some(TokenKind::LCurly) => object_value(parser),
        _ => {
            parser.push_err(create_err!(
                parser
                    .peek_data()
                    .unwrap_or_else(|| String::from("no further data")),
                "Expected a valid Value, got {}",
                parser
                    .peek_data()
                    .unwrap_or_else(|| String::from("no further data"))
            ));
        }
    }
}

pub(crate) fn list_value(parser: &mut Parser) {
    let list_guard = parser.start_node(SyntaxKind::LIST_VALUE);
    parser.bump(SyntaxKind::L_BRACK);
    match parser.peek() {
        Some(TokenKind::Node) => {
            value(parser);
            if let Some(TokenKind::RBracket) = parser.peek() {
                parser.bump(SyntaxKind::R_BRACK);
                list_guard.finish_node()
            } else {
                parser.push_err(create_err!(
                    parser
                        .peek_data()
                        .unwrap_or_else(|| String::from("no further data")),
                    "Expected a closing ] to follow a List Value, got {}",
                    parser
                        .peek_data()
                        .unwrap_or_else(|| String::from("no further data"))
                ));
            }
        }
        Some(TokenKind::RBracket) => {}
        _ => {
            parser.push_err(create_err!(
                parser
                    .peek_data()
                    .unwrap_or_else(|| String::from("no further data")),
                "Expected a List Value, got {}",
                parser
                    .peek_data()
                    .unwrap_or_else(|| String::from("no further data"))
            ));
        }
    }
}
