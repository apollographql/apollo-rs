use crate::{
    parser::grammar::{description, directive, name, ty, value},
    Parser, SyntaxKind, TokenKind, S, T,
};

/// See: https://spec.graphql.org/draft/#InputObjectTypeDefinition
///
/// *InputObjectTypeDefinition*:
///     Description<sub>opt</sub> **input** Name Directives<sub>\[Const\] opt</sub> InputFieldsDefinition<sub>opt</sub>
/// ```
pub(crate) fn input_object_type_definition(p: &mut Parser) {
    let _g = p.start_node(SyntaxKind::INPUT_OBJECT_TYPE_DEFINITION);

    if let Some(TokenKind::StringValue) = p.peek() {
        description::description(p);
    }

    if let Some("input") = p.peek_data().as_deref() {
        p.bump(SyntaxKind::input_KW);
    }

    match p.peek() {
        Some(TokenKind::Name) => name::name(p),
        _ => p.err("expected a Name"),
    }

    if let Some(T![@]) = p.peek() {
        directive::directives(p);
    }

    if let Some(T!['{']) = p.peek() {
        input_fields_definition(p);
    }
}

/// See: https://spec.graphql.org/draft/#InputObjectTypeExtension
///
/// *InputObjectTypeExtension*:
///     **extend** **input** Name Directives<sub>\[Const\] opt</sub> InputFieldsDefinition
///     **extend** **input** Name Directives<sub>\[Const\]</sub>
pub(crate) fn input_object_type_extension(p: &mut Parser) {
    let _g = p.start_node(SyntaxKind::INPUT_OBJECT_TYPE_EXTENSION);
    p.bump(SyntaxKind::extend_KW);
    p.bump(SyntaxKind::input_KW);

    let mut meets_requirements = false;

    match p.peek() {
        Some(TokenKind::Name) => name::name(p),
        _ => p.err("expected a Name"),
    }

    if let Some(T![@]) = p.peek() {
        meets_requirements = true;
        directive::directives(p);
    }

    if let Some(T!['{']) = p.peek() {
        meets_requirements = true;
        input_fields_definition(p);
    }

    if !meets_requirements {
        p.err("expected Directives or an Input Fields Definition");
    }
}

/// See: https://spec.graphql.org/draft/#InputFieldsDefinition
///
/// *InputFieldsDefinition*:
///     **{** InputValueDefinition<sub>list</sub> **}**
pub(crate) fn input_fields_definition(p: &mut Parser) {
    let _g = p.start_node(SyntaxKind::INPUT_FIELDS_DEFINITION);
    p.bump(S!['{']);
    input_value_definition(p, false);
    p.expect(T!['}'], S!['}']);
}

/// See: https://spec.graphql.org/draft/#InputValueDefinition
///
/// *InputValueDefinition*:
///     Description<sub>opt</sub> Name **:** Type DefaultValue<sub>opt</sub> Directives<sub>\[Const\] opt</sub>
pub(crate) fn input_value_definition(p: &mut Parser, is_input: bool) {
    if let Some(TokenKind::Name | TokenKind::StringValue) = p.peek() {
        let guard = p.start_node(SyntaxKind::INPUT_VALUE_DEFINITION);

        if let Some(TokenKind::StringValue) = p.peek() {
            description::description(p);
        }

        name::name(p);

        if let Some(T![:]) = p.peek() {
            p.bump(S![:]);
            match p.peek() {
                Some(TokenKind::Name) | Some(T!['[']) => {
                    ty::ty(p);
                    if let Some(T![=]) = p.peek() {
                        value::default_value(p);
                    }

                    if let Some(T![@]) = p.peek() {
                        directive::directives(p);
                    }

                    if p.peek().is_some() {
                        guard.finish_node();
                        return input_value_definition(p, true);
                    }
                }
                _ => p.err("expected a Type"),
            }
        } else {
            p.err("expected a Name");
        }
    }
    // TODO @lrlna: this can be simplified a little bit, and follow the pattern of FieldDefinition
    if !is_input {
        p.err("expected an Input Value Definition");
    }
}
