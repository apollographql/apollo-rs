use crate::parser::grammar::{input, name, value};
use crate::{Parser, SyntaxKind, TokenKind, S, T};

/// See: https://spec.graphql.org/draft/#Argument
///
/// *Argument*<sub>\[Const\]</sub>:
///    Name **:** Value<sub>\[?Const\]</sub>
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
    if !is_argument {
        p.err("expected an Argument");
    }
}

/// See: https://spec.graphql.org/draft/#Arguments
///
/// *Arguments*<sub>\[Const\]</sub>:
///    **(** Argument<sub>\[?Const\] list</sub> **)**
pub(crate) fn arguments(p: &mut Parser) {
    let _g = p.start_node(SyntaxKind::ARGUMENTS);
    p.bump(S!['(']);
    argument(p, false);
    p.expect(T![')'], S![')']);
}

/// See: https://spec.graphql.org/draft/#ArgumentsDefinition
///
/// *ArgumentsDefinition*:
///     **(** InputValueDefinition<sub>list</sub> **)**
pub(crate) fn arguments_definition(p: &mut Parser) {
    let _g = p.start_node(SyntaxKind::ARGUMENTS);
    p.bump(S!['(']);
    input::input_value_definition(p, false);
    p.expect(T![')'], S![')']);
}
