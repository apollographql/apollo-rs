use crate::parser::grammar::{argument, description, directive, name, selection, ty};
use crate::{Parser, SyntaxKind, TokenKind, S, T};

/// See: https://spec.graphql.org/June2018/#Field
///
/// ```txt
/// Field
///     Alias(opt) Name Arguments(opt) Directives(opt) SelectionSet(opt)
/// ```
pub(crate) fn field(p: &mut Parser) {
    let guard = p.start_node(SyntaxKind::FIELD);
    if let Some(TokenKind::Name) = p.peek() {
        if let Some(T![:]) = p.peek_n(2) {
            name::alias(p)
        }
        name::name(p)
    } else {
        p.err("expected a Name");
    }
    match p.peek() {
        Some(T!['(']) => argument::arguments(p),
        Some(T![@]) => directive::directives(p),
        Some(T!['{']) => selection::selection_set(p),
        Some(TokenKind::Name) => {
            guard.finish_node();
            field(p)
        }
        Some(T!['}']) => {
            guard.finish_node();
        }
        _ => guard.finish_node(),
    }
}

/// See: https://spec.graphql.org/June2018/#FieldsDefinition
///
/// ```txt
/// FieldsDefinition
///     { FieldDefinition[list] }
/// ```
pub(crate) fn fields_definition(p: &mut Parser) {
    let _g = p.start_node(SyntaxKind::FIELDS_DEFINITION);
    p.bump(S!['{']);
    field_definition(p);
    p.expect(T!['}'], S!['}']);
}

/// See: https://spec.graphql.org/June2018/#FieldDefinition
///
/// ```txt
/// FieldDefinition
///     Description[opt] Name ArgumentsDefinition[opt] : Type Directives[Const][opt]
/// ```
pub(crate) fn field_definition(p: &mut Parser) {
    if let Some(TokenKind::Name | TokenKind::StringValue) = p.peek() {
        let guard = p.start_node(SyntaxKind::FIELD_DEFINITION);

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
                        guard.finish_node();
                        return field_definition(p);
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

    if let Some(T!['}']) = p.peek() {
        return;
    }
}
