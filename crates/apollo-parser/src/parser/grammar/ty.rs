use std::collections::VecDeque;

use crate::{Parser, SyntaxKind, Token, TokenKind, S, T};

use crate::parser::grammar::name;

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
pub(crate) fn ty(p: &mut Parser) {
    let mut types = parse(p);

    process(&mut types, p);

    return;

    fn parse(p: &mut Parser) -> VecDeque<(SyntaxKind, Token)> {
        let token = p.pop();
        let mut types = match token.kind() {
            T!['['] => {
                let mut types = parse(p);
                types.push_front((S!['['], token));
                if let Some(T![']']) = p.peek() {
                    types.push_back((S![']'], p.pop()));
                }
                types
            }
            TokenKind::Name => {
                let mut types = VecDeque::new();
                types.push_back((SyntaxKind::NAMED_TYPE, token));
                types
            }
            // TODO(@lrlna): this should not panic
            token => panic!("unexpected token, {:?}", token),
        };

        if let Some(T![!]) = p.peek() {
            types.push_front((SyntaxKind::NON_NULL_TYPE, p.pop()));
        }

        types
    }

    fn process(types: &mut VecDeque<(SyntaxKind, Token)>, p: &mut Parser) {
        match types.pop_front() {
            Some((kind @ S!['['], token)) => {
                let _ty_guard = p.start_node(SyntaxKind::TYPE);
                let _list_guard = p.start_node(SyntaxKind::LIST_TYPE);
                p.push_ast(kind, token);
                process(types, p);
                if let Some((_kind @ S![']'], _token)) = peek(types) {
                    process(types, p);
                }
            }
            Some((kind @ SyntaxKind::NON_NULL_TYPE, _)) => {
                let _ty_guard = p.start_node(SyntaxKind::TYPE);
                let _non_null_guard = p.start_node(kind);
                process(types, p);
            }
            Some((SyntaxKind::NAMED_TYPE, _)) => {
                let _ty_guard = p.start_node(SyntaxKind::TYPE);
                p.start_node(SyntaxKind::NAMED_TYPE).finish_node();
            }
            Some((kind @ S![']'], token)) => {
                p.push_ast(kind, token);
            }
            _ => p.err("Internal apollo-parser error: unexpected when creating a Type"),
        }
    }
}

/// See: https://spec.graphql.org/June2018/#NamedType
///
/// ```txt
/// NamedType
///     Name
/// ```
pub(crate) fn named_type(p: &mut Parser) {
    let _guard = p.start_node(SyntaxKind::NAMED_TYPE);
    name::name(p);
}

fn peek<T>(target: &VecDeque<T>) -> Option<&T> {
    match target.len() {
        0 => None,
        len => target.get(len - 1),
    }
}
