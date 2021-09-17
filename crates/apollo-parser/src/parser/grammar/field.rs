use crate::parser::grammar::{argument, directive, name, selection, ty};
use crate::{create_err, Parser, SyntaxKind, TokenKind, S, T};

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
        p.push_err(create_err!(
            p.peek_data()
                .unwrap_or_else(|| String::from("no further data")),
            "Expected Field to have a Name, got {}",
            p.peek_data()
                .unwrap_or_else(|| String::from("no further data"))
        ));
    }
    match p.peek() {
        Some(T!['(']) => argument::arguments(p),
        Some(T![@]) => directive::directives(p),
        Some(T!['{']) => selection::selection_set(p),
        Some(T![,]) => {
            guard.finish_node();
            p.bump(S![,]);
            field(p)
        }
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
    let _guard = p.start_node(SyntaxKind::FIELDS_DEFINITION);
    let _guard = p.start(SyntaxKind::FIELDS_DEFINITION);
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
    if let Some(TokenKind::Name) = p.peek() {
        let guard = p.start_node(SyntaxKind::FIELD_DEFINITION);
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
                    p.push_err(create_err!(
                        p.peek_data().unwrap(),
                        "Expected InputValue definition to have a Type, got {}",
                        p.peek_data().unwrap()
                    ));
                }
            }
        } else {
            p.push_err(create_err!(
                p.peek_data().unwrap(),
                "Expected Field Definition to have a Type, got {}",
                p.peek_data().unwrap()
            ));
        }
    }

    if let Some(T![,]) = p.peek() {
        p.bump(S![,]);
        return field_definition(p);
    }

    if let Some(T!['}']) = p.peek() {
        return;
    }
}
