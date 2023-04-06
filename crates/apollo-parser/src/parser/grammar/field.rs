#![allow(clippy::needless_return)]

use crate::{
    parser::grammar::{argument, description, directive, name, selection, ty},
    Parser, SyntaxKind, TokenKind, S, T,
};

/// See: https://spec.graphql.org/October2021/#Field
///
/// *Field*:
///     Alias? Name Arguments? Directives? SelectionSet?
pub(crate) fn field(p: &mut Parser) {
    let _guard = p.start_node(SyntaxKind::FIELD);

    if let Some(TokenKind::Name) = p.peek() {
        if let Some(T![:]) = p.peek_n(2) {
            name::alias(p)
        }
        name::name(p)
    } else {
        p.err("expected a Name");
    }

    if let Some(T!['(']) = p.peek() {
        argument::arguments(p);
    }

    if let Some(T![@]) = p.peek() {
        directive::directives(p);
    }

    if let Some(T!['{']) = p.peek() {
        selection::selection_set(p);
    }
}

/// See: https://spec.graphql.org/October2021/#FieldsDefinition
///
/// *FieldsDefinition*:
///     **{** FieldDefinition* **}**
pub(crate) fn fields_definition(p: &mut Parser) {
    let _g = p.start_node(SyntaxKind::FIELDS_DEFINITION);
    p.bump(S!['{']);
    while let Some(TokenKind::Name | TokenKind::StringValue) = p.peek() {
        // Guaranteed to eat at least one token if the next token is a Name or StringValue
        field_definition(p);
    }
    p.expect(T!['}'], S!['}']);
}

/// See: https://spec.graphql.org/October2021/#FieldDefinition
///
/// *FieldDefinition*:
///     Description? Name ArgumentsDefinition? **:** Type Directives?
pub(crate) fn field_definition(p: &mut Parser) {
    let _guard = p.start_node(SyntaxKind::FIELD_DEFINITION);

    if let Some(TokenKind::StringValue) = p.peek() {
        description::description(p);
    }

    name::name(p);

    if let Some(T!['(']) = p.peek() {
        argument::arguments_definition(p);
    }

    if let Some(T![:]) = p.peek() {
        p.bump(S![:]);
        match p.peek() {
            Some(TokenKind::Name) | Some(T!['[']) => {
                ty::ty(p);
                if let Some(T![@]) = p.peek() {
                    directive::directives(p);
                }
                if p.peek().is_some() {
                    return;
                }
            }
            _ => {
                p.err("expected a Type");
            }
        }
    } else {
        p.err("expected a type");
    }
}
