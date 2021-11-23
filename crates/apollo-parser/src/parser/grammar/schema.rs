use crate::{
    parser::grammar::{description, directive, operation},
    Parser, SyntaxKind, TokenKind, S, T,
};

/// See: https://spec.graphql.org/October2021/#SchemaDefinition
///
/// *SchemaDefinition*:
///     Description? **schema** Directives? **{** RootOperationTypeDefinition* **}**
pub(crate) fn schema_definition(p: &mut Parser) {
    let _g = p.start_node(SyntaxKind::SCHEMA_DEFINITION);

    if let Some(TokenKind::StringValue) = p.peek() {
        description::description(p);
    }

    if let Some("schema") = p.peek_data().as_deref() {
        p.bump(SyntaxKind::schema_KW);
    }

    if let Some(T![@]) = p.peek() {
        directive::directives(p);
    }

    if let Some(T!['{']) = p.peek() {
        operation::root_operation_type_definition(p, false);
        p.expect(T!['}'], S!['}']);
    } else {
        p.err("expected Root Operation Type Definition");
    }
}

/// See: https://spec.graphql.org/October2021/#SchemaExtension
///
/// *SchemaExtension*:
///     **extend** **schema** Directives? **{** RootOperationTypeDefinition* **}**
///     **extend** **schema** Directives
pub(crate) fn schema_extension(p: &mut Parser) {
    let _g = p.start_node(SyntaxKind::SCHEMA_EXTENSION);
    p.bump(SyntaxKind::extend_KW);
    p.bump(SyntaxKind::schema_KW);

    let mut meets_requirements = false;

    if let Some(T![@]) = p.peek() {
        meets_requirements = true;
        directive::directives(p);
    }

    if let Some(T!['{']) = p.peek() {
        meets_requirements = true;
        operation::root_operation_type_definition(p, false);
        p.expect(T!['}'], S!['}']);
    }

    if !meets_requirements {
        p.err("expected directives or Root Operation Type Definition");
    }
}
