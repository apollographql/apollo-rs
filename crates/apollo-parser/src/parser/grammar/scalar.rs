use crate::parser::grammar::value::Constness;
use crate::parser::grammar::{description, directive, name};
use crate::{Parser, SyntaxKind, TokenKind, T};

/// See: https://spec.graphql.org/October2021/#ScalarTypeDefinition
///
/// *ScalarTypeDefinition*:
///     Description? **scalar** Name Directives[Const]?
pub(crate) fn scalar_type_definition(p: &mut Parser) {
    let _g = p.start_node(SyntaxKind::SCALAR_TYPE_DEFINITION);

    if let Some(TokenKind::StringValue) = p.peek() {
        description::description(p);
    }

    if let Some("scalar") = p.peek_data().as_deref() {
        p.bump(SyntaxKind::scalar_KW);
    }

    match p.peek() {
        Some(TokenKind::Name) => name::name(p),
        _ => p.err("expected a Name"),
    }

    if let Some(T![@]) = p.peek() {
        directive::directives(p, Constness::Const);
    }
}

/// See: https://spec.graphql.org/October2021/#ScalarTypeExtension
///
/// *ScalarTypeExtension*:
///     **extend** **scalar** Name Directives[Const]
pub(crate) fn scalar_type_extension(p: &mut Parser) {
    let _g = p.start_node(SyntaxKind::SCALAR_TYPE_EXTENSION);
    p.bump(SyntaxKind::extend_KW);
    p.bump(SyntaxKind::scalar_KW);

    match p.peek() {
        Some(TokenKind::Name) => name::name(p),
        _ => p.err("expected a Name"),
    }

    match p.peek() {
        Some(T![@]) => directive::directives(p, Constness::Const),
        _ => p.err("expected Directives"),
    }
}
