#![allow(clippy::needless_return)]

use crate::parser::grammar::value::Constness;
use crate::parser::grammar::{argument, description, directive, name, selection, ty};
use crate::{Parser, SyntaxKind, TokenKind, S, T};
use std::ops::ControlFlow;
use crate::parser::grammar::enum_::enum_value_definition;

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
        argument::arguments(p, Constness::NotConst);
    }

    if let Some(T![@]) = p.peek() {
        directive::directives(p, Constness::NotConst);
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

    match p.peek() {
        Some(TokenKind::Name | TokenKind::StringValue) => field_definition(p),
        _ => p.err("expected Field Definition"),
    }

    p.peek_while(|p, kind| match kind {
        TokenKind::Name | TokenKind::StringValue => {
            field_definition(p);
            ControlFlow::Continue(())
        }
        _ => ControlFlow::Break(()),
    });
    p.expect(T!['}'], S!['}']);
}

/// See: https://spec.graphql.org/October2021/#FieldDefinition
///
/// *FieldDefinition*:
///     Description? Name ArgumentsDefinition? **:** Type Directives[Const]?
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
                    directive::directives(p, Constness::Const);
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
