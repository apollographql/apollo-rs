use crate::parser::{name, ty, value};
use crate::{create_err, Parser, SyntaxKind, TokenKind};

/// See: https://spec.graphql.org/June2018/#InputValueDefinition
///
/// ```txt
/// InputValueDefinition
///     Description(opt) Name : Type DefaultValue(opt) Directives(const/opt)
/// ```
pub(crate) fn input_value_definition(parser: &mut Parser, is_input: bool) {
    if let Some(TokenKind::Node) = parser.peek() {
        let guard = parser.start_node(SyntaxKind::INPUT_VALUE_DEFINITION);
        name::name(parser);
        if let Some(TokenKind::Colon) = parser.peek() {
            parser.bump(SyntaxKind::COLON);
            match parser.peek() {
                Some(TokenKind::Node) | Some(TokenKind::LBracket) => {
                    ty::ty(parser);
                    if let Some(TokenKind::Eq) = parser.peek() {
                        value::default_value(parser);
                    }
                    if parser.peek().is_some() {
                        guard.finish_node();
                        return input_value_definition(parser, true);
                    }
                }
                _ => {
                    parser.push_err(create_err!(
                        parser.peek_data().unwrap(),
                        "Expected InputValue definition to have a Type, got {}",
                        parser.peek_data().unwrap()
                    ));
                }
            }
        } else {
            parser.push_err(create_err!(
                parser.peek_data().unwrap(),
                "Expected InputValue definition to have a Name, got {}",
                parser.peek_data().unwrap()
            ));
        }
    }
    if let Some(TokenKind::Comma) = parser.peek() {
        parser.bump(SyntaxKind::COMMA);
        return input_value_definition(parser, is_input);
    }
    // if we already have an input, can proceed without returning an error
    if !is_input {
        parser.push_err(create_err!(
            parser.peek_data().unwrap(),
            "Expected to have an InputValue definition, got {}",
            parser.peek_data().unwrap()
        ));
    }
}
