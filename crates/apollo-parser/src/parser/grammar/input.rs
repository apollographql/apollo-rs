use crate::{
    parser::grammar::{description, directive, name, ty, value},
    Parser, SyntaxKind, TokenKind, S, T,
};

/// See: https://spec.graphql.org/October2021/#InputObjectTypeDefinition
///
/// *InputObjectTypeDefinition*:
///     Description? **input** Name Directives? InputFieldsDefinition?
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

/// See: https://spec.graphql.org/October2021/#InputObjectTypeExtension
///
/// *InputObjectTypeExtension*:
///     **extend** **input** Name Directives? InputFieldsDefinition
///     **extend** **input** Name Directives
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

/// See: https://spec.graphql.org/October2021/#InputFieldsDefinition
///
/// *InputFieldsDefinition*:
///     **{** InputValueDefinition* **}**
pub(crate) fn input_fields_definition(p: &mut Parser) {
    let _g = p.start_node(SyntaxKind::INPUT_FIELDS_DEFINITION);
    p.bump(S!['{']);
    input_value_definition(p, false);
    p.expect(T!['}'], S!['}']);
}

/// See: https://spec.graphql.org/October2021/#InputValueDefinition
///
/// *InputValueDefinition*:
///     Description? Name **:** Type DefaultValue? Directives?
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
                        // TODO: use a loop instead of recursion
                        if p.recursion_limit.check_and_increment() {
                            p.limit_err("parser recursion limit reached");
                            return;
                        }
                        input_value_definition(p, true);
                        p.recursion_limit.decrement();
                        return;
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
