use crate::{
    parser::grammar::{directive, name, selection, ty},
    Parser, SyntaxKind, TokenKind, S, T,
};

/// See: https://spec.graphql.org/October2021/#FragmentDefinition
///
/// *FragmentDefinition*:
///     **fragment** FragmentName TypeCondition Directives? SelectionSet
pub(crate) fn fragment_definition(p: &mut Parser) {
    let _g = p.start_node(SyntaxKind::FRAGMENT_DEFINITION);
    p.bump(SyntaxKind::fragment_KW);

    fragment_name(p);
    type_condition(p);

    if let Some(T![@]) = p.peek() {
        directive::directives(p);
    }

    p.recursion_limit.reset();
    match p.peek() {
        Some(T!['{']) => selection::selection_set(p),
        _ => p.err("expected a Selection Set"),
    }
}

/// See: https://spec.graphql.org/October2021/#FragmentName
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

/// See: https://spec.graphql.org/October2021/#TypeCondition
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
                p.err("exptected 'on'");
            }
            ty::named_type(p)
        }
        _ => p.err("expected Type Condition"),
    }
}

/// See: https://spec.graphql.org/October2021/#InlineFragment
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
        directive::directives(p);
    }

    p.recursion_limit.reset();
    match p.peek() {
        Some(T!['{']) => selection::selection_set(p),
        _ => p.err("expected Selection Set"),
    }
}

/// See: https://spec.graphql.org/October2021/#FragmentSpread
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
        directive::directives(p);
    }
}
