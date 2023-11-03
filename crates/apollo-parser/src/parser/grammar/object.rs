#![allow(clippy::needless_return)]

use crate::{
    parser::grammar::{description, directive, field, name, ty},
    Parser, SyntaxKind, TokenKind, S, T,
};

/// See: https://spec.graphql.org/October2021/#ObjectTypeDefinition
///
/// *ObjectTypeDefinition*:
///     Description? **type** Name ImplementsInterfaces? Directives? FieldsDefinition?
pub(crate) fn object_type_definition(p: &mut Parser) {
    let _g = p.start_node(SyntaxKind::OBJECT_TYPE_DEFINITION);

    if let Some(TokenKind::StringValue) = p.peek() {
        description::description(p);
    }

    if let Some("type") = p.peek_data().as_deref() {
        p.bump(SyntaxKind::type_KW);
    }

    match p.peek() {
        Some(TokenKind::Name) => name::name(p),
        _ => p.err("expected a name"),
    }

    if let Some(TokenKind::Name) = p.peek() {
        if p.peek_data().unwrap() == "implements" {
            implements_interfaces(p);
        } else {
            p.err("unexpected Name");
        }
    }

    if let Some(T![@]) = p.peek() {
        directive::directives(p);
    }

    if let Some(T!['{']) = p.peek() {
        field::fields_definition(p);
    }
}

/// See: https://spec.graphql.org/October2021/#ObjectTypeExtension
///
/// *ObjectTypeExtension*:
///     **extend** **type** Name ImplementsInterfaces? Directives? FieldsDefinition
///     **extend** **type** Name ImplementsInterfaces? Directives?
///     **extend** **type** Name ImplementsInterfaces
pub(crate) fn object_type_extension(p: &mut Parser) {
    let _g = p.start_node(SyntaxKind::OBJECT_TYPE_EXTENSION);
    p.bump(SyntaxKind::extend_KW);
    p.bump(SyntaxKind::type_KW);

    // Use this variable to see if any of ImplementsInterfacs, Directives or
    // FieldsDefinitions is provided. If none are present, we push an error.
    let mut meets_requirements = false;

    match p.peek() {
        Some(TokenKind::Name) => name::name(p),
        _ => p.err("expected a Name"),
    }

    if let Some("implements") = p.peek_data().as_deref() {
        meets_requirements = true;
        implements_interfaces(p);
    }

    if let Some(T![@]) = p.peek() {
        meets_requirements = true;
        directive::directives(p)
    }

    if let Some(T!['{']) = p.peek() {
        meets_requirements = true;
        field::fields_definition(p)
    }

    if !meets_requirements {
        p.err("expected an Implements Interface, Directives or a Fields Definition");
    }
}

/// See: https://spec.graphql.org/October2021/#ImplementsInterfaces
///
/// *ImplementsInterfaces*:
///     **implements** **&**? NamedType
///     ImplementsInterfaces **&** NamedType
pub(crate) fn implements_interfaces(p: &mut Parser) {
    let _g = p.start_node(SyntaxKind::IMPLEMENTS_INTERFACES);
    p.bump(SyntaxKind::implements_KW);

    if let Some(T![&]) = p.peek() {
        p.bump(S![&]);
    }

    if let Some(TokenKind::Name) = p.peek() {
        ty::named_type(p);
    } else {
        p.err("expected an Interface name");
    }

    while let Some(T![&]) = p.peek() {
        p.bump(S![&]);
        ty::named_type(p);
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::cst;

    #[test]
    fn object_type_definition() {
        let input = "
type Business implements NamedEntity & ValuedEntity & CatEntity {
  name: String
}";
        let parser = Parser::new(input);
        let cst = parser.parse();
        assert_eq!(0, cst.errors().len());

        let doc = cst.document();

        for def in doc.definitions() {
            if let cst::Definition::ObjectTypeDefinition(interface_type) = def {
                assert_eq!(interface_type.name().unwrap().text(), "Business");
                for implements_interfaces in interface_type
                    .implements_interfaces()
                    .unwrap()
                    .named_types()
                {
                    // NamedEntity ValuedEntity CatEntity
                    println!("{}", implements_interfaces.name().unwrap().text());
                }
            }
        }
    }
}
