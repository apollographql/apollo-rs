use crate::{
    parser::grammar::{description, directive, field, name, object},
    Parser, SyntaxKind, TokenKind, T,
};

/// See: https://spec.graphql.org/October2021/#InterfaceTypeDefinition
///
/// *InterfaceTypeDefinition*:
///     Description? **interface** Name ImplementsInterface? Directives? FieldsDefinition?
pub(crate) fn interface_type_definition(p: &mut Parser) {
    let _g = p.start_node(SyntaxKind::INTERFACE_TYPE_DEFINITION);

    if let Some(TokenKind::StringValue) = p.peek() {
        description::description(p);
    }

    if let Some("interface") = p.peek_data().as_deref() {
        p.bump(SyntaxKind::interface_KW);
    }

    match p.peek() {
        Some(TokenKind::Name) => name::name(p),
        _ => p.err("expected a Name"),
    }

    if let Some("implements") = p.peek_data().as_deref() {
        object::implements_interfaces(p);
    }

    if let Some(T![@]) = p.peek() {
        directive::directives(p);
    }

    if let Some(T!['{']) = p.peek() {
        field::fields_definition(p);
    }
}

/// See: https://spec.graphql.org/October2021/#InterfaceTypeExtension
///
/// *InterfaceTypeExtension*:
///     **extend** **interface** Name ImplementsInterface? Directives? FieldsDefinition
///     **extend** **interface** Name ImplementsInterface? Directives
///     **extend** **interface** Name ImplementsInterface
pub(crate) fn interface_type_extension(p: &mut Parser) {
    let _g = p.start_node(SyntaxKind::INTERFACE_TYPE_EXTENSION);
    p.bump(SyntaxKind::extend_KW);
    p.bump(SyntaxKind::interface_KW);

    let mut meets_requirements = false;

    match p.peek() {
        Some(TokenKind::Name) => name::name(p),
        _ => p.err("expected a Name"),
    }

    if let Some("implements") = p.peek_data().as_deref() {
        meets_requirements = true;
        object::implements_interfaces(p);
    }

    if let Some(T![@]) = p.peek() {
        meets_requirements = true;
        directive::directives(p);
    }

    if let Some(T!['{']) = p.peek() {
        meets_requirements = true;
        field::fields_definition(p);
    }

    if !meets_requirements {
        p.err("exptected an Implements Interfaces, Directives, or a Fields Definition");
    }
}
