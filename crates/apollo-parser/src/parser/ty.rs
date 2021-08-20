use std::collections::VecDeque;

use crate::{create_err, Parser, SyntaxKind, Token, TokenKind};

use crate::parser::name;

/// See: https://spec.graphql.org/June2018/#InputValueDefinition
///
/// ```txt
/// Type
///     NamedType
///     ListType
///         [ Type ]
///     NonNullType
///         NamedType!
///         ListType!
/// ```
// NOTE(lrlna): Because Type cannot be parsed in a typical LR fashion, the
// following parsing rule does not follow the same pattern as all other parsing
// rules in this library. The parent node type is determined based on what its
// last possible is a NonNullType.
//
// To make this work, we first collect all types in a double ended queue, and
// unwrap them once the last possible child has been parsed. Nodes are then
// created in the processing stage of this parsing rule.
pub(crate) fn ty(parser: &mut Parser) {
    let mut types = parse(parser);

    process(&mut types, parser);

    return;

    fn parse(parser: &mut Parser) -> VecDeque<(SyntaxKind, Token)> {
        let token = parser.pop();
        let mut types = match token.kind() {
            TokenKind::LBracket => {
                let mut types = parse(parser);
                types.push_front((SyntaxKind::L_BRACK, token));
                if let Some(TokenKind::RBracket) = parser.peek() {
                    types.push_back((SyntaxKind::R_BRACK, parser.pop()));
                }
                types
            }
            TokenKind::Node => {
                let mut types = VecDeque::new();
                types.push_back((SyntaxKind::NAMED_TYPE, token));
                types
            }
            // TODO(@lrlna): this should not panic
            token => panic!("unexpected token, {:?}", token),
        };

        if let Some(TokenKind::Bang) = parser.peek() {
            types.push_front((SyntaxKind::NON_NULL_TYPE, parser.pop()));
        }

        types
    }

    fn process(types: &mut VecDeque<(SyntaxKind, Token)>, parser: &mut Parser) {
        match types.pop_front() {
            Some((kind @ SyntaxKind::L_BRACK, token)) => {
                let _ty_guard = parser.start_node(SyntaxKind::TYPE);
                let _list_guard = parser.start_node(SyntaxKind::LIST_TYPE);
                parser.push_ast(kind, token);
                process(types, parser);
                if let Some((_kind @ SyntaxKind::R_BRACK, _token)) = peek(types) {
                    process(types, parser);
                }
            }
            Some((kind @ SyntaxKind::NON_NULL_TYPE, _)) => {
                let _ty_guard = parser.start_node(SyntaxKind::TYPE);
                let _non_null_guard = parser.start_node(kind);
                process(types, parser);
            }
            Some((SyntaxKind::NAMED_TYPE, _)) => {
                let _ty_guard = parser.start_node(SyntaxKind::TYPE);
                parser.start_node(SyntaxKind::NAMED_TYPE).finish_node();
            }
            Some((kind @ SyntaxKind::R_BRACK, token)) => {
                parser.push_ast(kind, token);
            }
            _ => {
                parser.push_err(create_err!(
                    parser
                        .peek_data()
                        .unwrap_or_else(|| String::from("no further data")),
                    "Internal apollo-parser error, {} token was not expected when creating a Type",
                    parser
                        .peek_data()
                        .unwrap_or_else(|| String::from("no further data"))
                ));
            }
        }
    }
}

/// See: https://spec.graphql.org/June2018/#NamedType
///
/// ```txt
/// NamedType
///     Name
/// ```
pub(crate) fn named_type(parser: &mut Parser) -> Result<(), crate::Error> {
    let _guard = parser.start_node(SyntaxKind::NAMED_TYPE);
    name::name(parser);
    Ok(())
}

fn peek<T>(target: &VecDeque<T>) -> Option<&T> {
    match target.len() {
        0 => None,
        len => target.get(len - 1),
    }
}
