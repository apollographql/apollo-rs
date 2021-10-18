use crate::{
    parser::grammar::{argument, description, directive, name, selection, ty},
    Parser, SyntaxKind, TokenKind, S, T,
};

/// See: https://spec.graphql.org/draft/#Field
///
/// *Field*:
///     Alias<sub>opt</sub> Name Arguments<sub>opt</sub> Directives<sub>opt</sub> SelectionSet<sub>opt</sub>
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

    if let Some(T!['(']) = p.peek() {
        argument::arguments(p);
    }

    if let Some(T![@]) = p.peek() {
        directive::directives(p);
    }

    if let Some(T!['{']) = p.peek() {
        selection::selection_set(p);
    }

    match p.peek() {
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

/// See: https://spec.graphql.org/draft/#FieldsDefinition
///
/// *FieldsDefinition*:
///     **{** FieldDefinition<sub>list</sub> **}**
pub(crate) fn fields_definition(p: &mut Parser) {
    let _g = p.start_node(SyntaxKind::FIELDS_DEFINITION);
    p.bump(S!['{']);
    field_definition(p);
    p.expect(T!['}'], S!['}']);
}

/// See: https://spec.graphql.org/draft/#FieldDefinition
///
/// *FieldDefinition*:
///     Description<sub>opt</sub> Name ArgumentsDefinition<sub>opt</sub> **:** Type Directives<sub>\[Const\] opt</sub>
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
