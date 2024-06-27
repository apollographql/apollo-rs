use crate::parser::grammar::description;
use crate::parser::grammar::directive;
use crate::parser::grammar::name;
use crate::parser::grammar::ty;
use crate::parser::grammar::value;
use crate::parser::grammar::value::Constness;
use crate::Parser;
use crate::SyntaxKind;
use crate::TokenKind;
use crate::S;
use crate::T;
use std::ops::ControlFlow;

/// See: https://spec.graphql.org/October2021/#InputObjectTypeDefinition
///
/// *InputObjectTypeDefinition*:
///     Description? **input** Name Directives[Const]? InputFieldsDefinition?
/// ```
pub(crate) fn input_object_type_definition(p: &mut Parser) {
    let _g = p.start_node(SyntaxKind::INPUT_OBJECT_TYPE_DEFINITION);

    if let Some(TokenKind::StringValue) = p.peek() {
        description::description(p);
    }

    if let Some("input") = p.peek_data() {
        p.bump(SyntaxKind::input_KW);
    }

    match p.peek() {
        Some(TokenKind::Name) => name::name(p),
        _ => p.err("expected a Name"),
    }

    if let Some(T![@]) = p.peek() {
        directive::directives(p, Constness::Const);
    }

    if let Some(T!['{']) = p.peek() {
        input_fields_definition(p);
    }
}

/// See: https://spec.graphql.org/October2021/#InputObjectTypeExtension
///
/// *InputObjectTypeExtension*:
///     **extend** **input** Name Directives[Const]? InputFieldsDefinition
///     **extend** **input** Name Directives[Const]
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
        directive::directives(p, Constness::Const);
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
    if let Some(TokenKind::Name | TokenKind::StringValue) = p.peek() {
        input_value_definition(p);
    } else {
        p.err("expected an Input Value Definition");
    }
    p.peek_while(|p, kind| {
        if matches!(kind, TokenKind::Name | TokenKind::StringValue) {
            input_value_definition(p);
            ControlFlow::Continue(())
        } else {
            ControlFlow::Break(())
        }
    });

    p.expect(T!['}'], S!['}']);
}

/// See: https://spec.graphql.org/October2021/#InputValueDefinition
///
/// *InputValueDefinition*:
///     Description? Name **:** Type DefaultValue? Directives[Const]?
pub(crate) fn input_value_definition(p: &mut Parser) {
    let _guard = p.start_node(SyntaxKind::INPUT_VALUE_DEFINITION);

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
                    directive::directives(p, Constness::Const);
                }
            }
            _ => p.err("expected a Type"),
        }
    } else {
        p.err("expected a Name");
    }
}
