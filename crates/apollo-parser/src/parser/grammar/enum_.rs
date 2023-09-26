#![allow(clippy::needless_return)]

use crate::{
    parser::grammar::{description, directive, name, value},
    Parser, SyntaxKind, TokenKind, S, T,
};

/// See: https://spec.graphql.org/October2021/#EnumTypeDefinition
///
/// *EnumTypeDefinition*:
///     Description? **enum** Name Directives? EnumValuesDefinition?
pub(crate) fn enum_type_definition(p: &mut Parser) {
    let _g = p.start_node(SyntaxKind::ENUM_TYPE_DEFINITION);

    if let Some(TokenKind::StringValue) = p.peek() {
        description::description(p);
    }

    if let Some("enum") = p.peek_data().as_deref() {
        p.bump(SyntaxKind::enum_KW);
    }

    match p.peek() {
        Some(TokenKind::Name) => name::name(p),
        _ => p.err("expected a Name"),
    }

    if let Some(T![@]) = p.peek() {
        directive::directives(p);
    }

    if let Some(T!['{']) = p.peek() {
        enum_values_definition(p);
    }
}

/// See: https://spec.graphql.org/October2021/#EnumTypeExtension
///
// *EnumTypeExtension*:
///    **extend** **enum** Name Directives? EnumValuesDefinition
///    **extend** **enum** Name Directives?
pub(crate) fn enum_type_extension(p: &mut Parser) {
    let _g = p.start_node(SyntaxKind::ENUM_TYPE_EXTENSION);
    p.bump(SyntaxKind::extend_KW);
    p.bump(SyntaxKind::enum_KW);

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
        enum_values_definition(p);
    }

    if !meets_requirements {
        p.err("expected Directive or Enum Values Definition");
    }
}

/// See: https://spec.graphql.org/October2021/#EnumValuesDefinition
///
/// *EnumValuesDefinition*:
///     **{** EnumValueDefinition* **}**
pub(crate) fn enum_values_definition(p: &mut Parser) {
    let _g = p.start_node(SyntaxKind::ENUM_VALUES_DEFINITION);
    p.bump(S!['{']);

    match p.peek() {
        Some(TokenKind::Name | TokenKind::StringValue) => enum_value_definition(p),
        _ => p.err("expected Enum Value Definition"),
    }

    p.expect(T!['}'], S!['}']);
}

/// See: https://spec.graphql.org/October2021/#EnumValueDefinition
///
/// *EnumValueDefinition*:
///     Description? EnumValue Directives?
pub(crate) fn enum_value_definition(p: &mut Parser) {
    if let Some(TokenKind::Name | TokenKind::StringValue) = p.peek() {
        let guard = p.start_node(SyntaxKind::ENUM_VALUE_DEFINITION);

        if let Some(TokenKind::StringValue) = p.peek() {
            description::description(p);
        }

        value::enum_value(p);

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
            enum_value_definition(p);
            p.recursion_limit.decrement();
            return;
        }
    }

    if let Some(T!['}']) = p.peek() {
        return;
    }
}
