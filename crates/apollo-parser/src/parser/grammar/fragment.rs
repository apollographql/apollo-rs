use crate::parser::grammar::description;
use crate::parser::grammar::directive;
use crate::parser::grammar::name;
use crate::parser::grammar::selection;
use crate::parser::grammar::ty;
use crate::parser::grammar::value::Constness;
use crate::Parser;
use crate::SyntaxKind;
use crate::TokenKind;
use crate::S;
use crate::T;

/// See: https://spec.graphql.org/September2025/#sec-Language.Fragments
///
/// *FragmentDefinition*:
///     Description? **fragment** FragmentName TypeCondition Directives? SelectionSet
pub(crate) fn fragment_definition(p: &mut Parser) {
    let _g = p.start_node(SyntaxKind::FRAGMENT_DEFINITION);

    // Check for optional description
    if let Some(TokenKind::StringValue) = p.peek() {
        description::description(p);
    }

    p.bump(SyntaxKind::fragment_KW);

    fragment_name(p);
    type_condition(p);

    if let Some(T![@]) = p.peek() {
        directive::directives(p, Constness::NotConst);
    }

    match p.peek() {
        Some(T!['{']) => selection::selection_set(p),
        _ => p.err("expected a Selection Set"),
    }
}

/// See: https://spec.graphql.org/September2025/#sec-Language.Fragments
///
/// *FragmentName*:
///     Name *but not* **on**
pub(crate) fn fragment_name(p: &mut Parser) {
    let _g = p.start_node(SyntaxKind::FRAGMENT_NAME);
    match p.peek() {
        Some(TokenKind::Name) => {
            if p.peek_data().unwrap() == "on" {
                return p.err("Fragment Name cannot be 'on'");
            }
            name::name(p)
        }
        _ => p.err("expected Fragment Name"),
    }
}

/// See: https://spec.graphql.org/September2025/#sec-Language.Fragments
///
/// *TypeCondition*:
///     **on** NamedType
pub(crate) fn type_condition(p: &mut Parser) {
    let _g = p.start_node(SyntaxKind::TYPE_CONDITION);
    match p.peek() {
        Some(TokenKind::Name) => {
            if p.peek_data().unwrap() == "on" {
                p.bump(SyntaxKind::on_KW);
            } else {
                p.err("expected 'on'");
            }

            if let Some(TokenKind::Name) = p.peek() {
                ty::named_type(p)
            } else {
                p.err("expected a Name in Type Condition")
            }
        }
        _ => p.err("expected Type Condition"),
    }
}

/// See: https://spec.graphql.org/September2025/#sec-Language.Fragments
///
/// *InlineFragment*:
///     **...** TypeCondition? Directives? SelectionSet
pub(crate) fn inline_fragment(p: &mut Parser) {
    let _g = p.start_node(SyntaxKind::INLINE_FRAGMENT);
    p.bump(S![...]);

    if let Some(TokenKind::Name) = p.peek() {
        type_condition(p);
    }

    if let Some(T![@]) = p.peek() {
        directive::directives(p, Constness::NotConst);
    }

    match p.peek() {
        Some(T!['{']) => selection::selection_set(p),
        _ => p.err("expected Selection Set"),
    }
}

/// See: https://spec.graphql.org/September2025/#sec-Language.Fragments
///
/// *FragmentSpread*:
///     **...** FragmentName Directives?
pub(crate) fn fragment_spread(p: &mut Parser) {
    let _g = p.start_node(SyntaxKind::FRAGMENT_SPREAD);
    p.bump(S![...]);

    match p.peek() {
        Some(TokenKind::Name) => {
            fragment_name(p);
        }
        _ => p.err("expected a Name"),
    }

    if let Some(T![@]) = p.peek() {
        directive::directives(p, Constness::NotConst);
    }
}
