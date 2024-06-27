use crate::parser::grammar::description;
use crate::parser::grammar::directive;
use crate::parser::grammar::operation;
use crate::parser::grammar::ty;
use crate::parser::grammar::value::Constness;
use crate::Parser;
use crate::SyntaxKind;
use crate::TokenKind;
use crate::S;
use crate::T;

/// RootOperationTypeDefinition is used in a SchemaDefinition. Not to be confused
/// with OperationDefinition.
///
/// See: https://spec.graphql.org/October2021/#RootOperationTypeDefinition
///
/// *RootOperationTypeDefinition*:
///    OperationType **:** NamedType
fn root_operation_type_definition(p: &mut Parser) {
    let _guard = p.start_node(SyntaxKind::ROOT_OPERATION_TYPE_DEFINITION);
    operation::operation_type(p);
    if let Some(T![:]) = p.peek() {
        p.bump(S![:]);
        ty::named_type(p);
    } else {
        p.err("expected a Name Type");
    }
}

/// See: https://spec.graphql.org/October2021/#SchemaDefinition
///
/// *SchemaDefinition*:
///     Description? **schema** Directives[Const]? **{** RootOperationTypeDefinition* **}**
pub(crate) fn schema_definition(p: &mut Parser) {
    let _g = p.start_node(SyntaxKind::SCHEMA_DEFINITION);

    if let Some(TokenKind::StringValue) = p.peek() {
        description::description(p);
    }

    if let Some("schema") = p.peek_data() {
        p.bump(SyntaxKind::schema_KW);
    }

    if let Some(T![@]) = p.peek() {
        directive::directives(p, Constness::Const);
    }

    if let Some(T!['{']) = p.peek() {
        p.bump(S!['{']);

        let mut has_root_operation_types = false;
        p.peek_while_kind(TokenKind::Name, |p| {
            has_root_operation_types = true;
            root_operation_type_definition(p);
        });
        if !has_root_operation_types {
            p.err("expected Root Operation Type Definition");
        }

        p.expect(T!['}'], S!['}']);
    }
}

/// See: https://spec.graphql.org/October2021/#SchemaExtension
///
/// *SchemaExtension*:
///     **extend** **schema** Directives[Const]? **{** RootOperationTypeDefinition* **}**
///     **extend** **schema** Directives[Const]
pub(crate) fn schema_extension(p: &mut Parser) {
    let _g = p.start_node(SyntaxKind::SCHEMA_EXTENSION);
    p.bump(SyntaxKind::extend_KW);
    p.bump(SyntaxKind::schema_KW);

    let mut meets_requirements = false;

    if let Some(T![@]) = p.peek() {
        meets_requirements = true;
        directive::directives(p, Constness::Const);
    }

    if let Some(T!['{']) = p.peek() {
        p.bump(S!['{']);

        p.peek_while_kind(TokenKind::Name, |p| {
            meets_requirements = true;
            root_operation_type_definition(p);
        });

        p.expect(T!['}'], S!['}']);
    }

    if !meets_requirements {
        p.err("expected directives or Root Operation Type Definition");
    }
}
