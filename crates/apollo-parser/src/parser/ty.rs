use std::collections::VecDeque;

use crate::{format_err, NodeGuard, Parser, SyntaxKind, Token, TokenKind};

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
pub(crate) fn ty(parser: &mut Parser) -> Result<(), crate::Error> {
    let mut types = parse(parser);

    process(&mut types, parser);
    debug_assert_eq!(types.len(), 0);

    return Ok(());

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
            token => panic!("unexpected token {:?}", token),
        };

        if let Some(TokenKind::Bang) = parser.peek() {
            types.push_front((SyntaxKind::NON_NULL_TYPE, parser.pop()));
        }

        types
    }

    fn process(mut types: &mut VecDeque<(SyntaxKind, Token)>, parser: &mut Parser) {
        match types.pop_front() {
            Some((kind @ SyntaxKind::L_BRACK, token)) => {
                let guard = parser.start_node(SyntaxKind::LIST_TYPE);
                parser.push_ast(kind, token);
                process(types, parser);
                if let Some((kind @ SyntaxKind::R_BRACK, token)) = peek(types) {
                    process(types, parser);
                }
                guard.finish_node();
            }
            Some((kind @ SyntaxKind::NON_NULL_TYPE, _)) => {
                let guard = parser.start_node(kind);
                process(types, parser);
                guard.finish_node();
            }
            Some((SyntaxKind::NAMED_TYPE, _)) => {
                parser.start_node(SyntaxKind::NAMED_TYPE).finish_node();
            }
            Some((kind @ SyntaxKind::R_BRACK, token)) => {
                parser.push_ast(kind, token);
            }
            _ => unreachable!(),
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
    name::name(parser)?;
    Ok(())
}

fn peek<T>(target: &VecDeque<T>) -> Option<&T> {
    match target.len() {
        0 => None,
        len => target.get(len - 1),
    }
}
