use crate::parser::grammar::{input, name, value};
use crate::{create_err, Parser, SyntaxKind, TokenKind, S, T};

/// See: https://spec.graphql.org/June2018/#Argument
///
/// ```txt
/// Argument
///    Name : Value
/// ```
pub(crate) fn argument(p: &mut Parser, mut is_argument: bool) {
    if let Some(TokenKind::Name) = p.peek() {
        let guard = p.start_node(SyntaxKind::ARGUMENT);
        name::name(p);
        if let Some(T![:]) = p.peek() {
            p.bump(S![:]);
            value::value(p);
            is_argument = true;
            if p.peek().is_some() {
                guard.finish_node();
                return argument(p, is_argument);
            }
        }
    }
    if let Some(T![,]) = p.peek() {
        p.bump(S![,]);
        return argument(p, is_argument);
    }
    if !is_argument {
        p.push_err(create_err!(
            p.peek_data()
                .unwrap_or_else(|| String::from("no further data")),
            "Expected to have an Argument, got {}",
            p.peek_data()
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
pub(crate) fn arguments(p: &mut Parser) {
    let guard = p.start_node(SyntaxKind::ARGUMENTS);
    p.bump(S!['(']);
    argument(p, false);
    match p.peek() {
        Some(T![')']) => {
            p.bump(S![')']);
            guard.finish_node();
        }
        _ => p.push_err(create_err!(
            p.peek_data()
                .unwrap_or_else(|| String::from("no further data")),
            "Expected closing ')', got {}",
            p.peek_data()
                .unwrap_or_else(|| String::from("no further data"))
        )),
    }
}

/// See: https://spec.graphql.org/June2018/#ArgumentsDefinition
///
/// ```txt
/// ArgumentsDefinition
///     ( InputValueDefinition[list] )
/// ```
pub(crate) fn arguments_definition(p: &mut Parser) {
    let guard = p.start_node(SyntaxKind::ARGUMENTS);
    p.bump(S!['(']);
    input::input_value_definition(p, false);
    match p.peek() {
        Some(T![')']) => {
            p.bump(S![')']);
            guard.finish_node();
        }
        _ => p.push_err(create_err!(
            p.peek_data()
                .unwrap_or_else(|| String::from("no further data")),
            "Expected closing ')', got {}",
            p.peek_data()
                .unwrap_or_else(|| String::from("no further data"))
        )),
    }
}
