use crate::parser::grammar::description;
use crate::parser::grammar::directive;
use crate::parser::grammar::name;
use crate::parser::grammar::value::Constness;
use crate::Parser;
use crate::SyntaxKind;
use crate::TokenKind;
use crate::T;

/// See: https://spec.graphql.org/October2021/#ScalarTypeDefinition
///
/// *ScalarTypeDefinition*:
///     Description? **scalar** Name Directives[Const]?
pub(crate) fn scalar_type_definition(p: &mut Parser) {
    let _g = p.start_node(SyntaxKind::SCALAR_TYPE_DEFINITION);

    if let Some(TokenKind::StringValue) = p.peek() {
        description::description(p);
    }

    if let Some("scalar") = p.peek_data() {
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
