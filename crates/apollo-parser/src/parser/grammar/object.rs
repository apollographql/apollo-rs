use crate::parser::grammar::{directive, field, name};
use crate::{create_err, Parser, SyntaxKind, TokenKind, T};

/// See: https://spec.graphql.org/June2018/#ObjectTypeDefinition
///
/// ```txt
/// ObjectTypeDefinition
///     Description[opt] type Name ImplementsInterfaces[opt] Directives[Const][opt] FieldsDefinition[opt]
/// ```
pub(crate) fn object_type_definition(p: &mut Parser) {
    let _guard = p.start_node(SyntaxKind::OBJECT_TYPE_DEFINITION);
    p.bump(SyntaxKind::type_KW);

    match p.peek() {
        Some(TokenKind::Name) => name::name(p),
        _ => {
            p.push_err(create_err!(
                p.peek_data()
                    .unwrap_or_else(|| String::from("no further data")),
                "Expected Object Type Definition to have a Name, got {}",
                p.peek_data()
                    .unwrap_or_else(|| String::from("no further data")),
            ));
        }
    }
    if let Some(TokenKind::Name) = p.peek() {
        if p.peek_data().unwrap() == "implements" {
            implements_interfaces(p, false);
        } else {
            p.push_err(create_err!(
                p.peek_data()
                    .unwrap_or_else(|| String::from("no further data")),
                "Unexpected Name in Object Type Definition, {}",
                p.peek_data()
                    .unwrap_or_else(|| String::from("no further data")),
            ));
        }
    }

    if let Some(T![@]) = p.peek() {
        directive::directives(p);
    }

    if let Some(T!['{']) = p.peek() {
        field::fields_definition(p);
    }
}

/// See: https://spec.graphql.org/June2018/#ObjectTypeExtension
///
/// ```txt
/// ObjectTypeExtension
///     extend type Name ImplementsInterfaces[opt] Directives[Const][opt] FieldsDefinition
///     extend type Name ImplementsInterfaces[opt] Directives[Const]
///     extend type Name ImplementsInterfaces
/// ```
pub(crate) fn object_type_extension(p: &mut Parser) {
    let _guard = p.start_node(SyntaxKind::OBJECT_TYPE_EXTENSION);
    p.bump(SyntaxKind::extend_KW);
    p.bump(SyntaxKind::type_KW);

    // Use this variable to see if any of ImplementsInterfacs, Directives or
    // FieldsDefinitions is provided. If none are present, we push an error.
    let mut meets_requirements = false;

    match p.peek() {
        Some(TokenKind::Name) => name::name(p),
        _ => {
            p.push_err(create_err!(
                p.peek_data()
                    .unwrap_or_else(|| String::from("no further data")),
                "Expected Object Type Extension to have a Name, got {}",
                p.peek_data()
                    .unwrap_or_else(|| String::from("no further data")),
            ));
        }
    }
    if let Some(TokenKind::Name) = p.peek() {
        if p.peek_data().unwrap() == "implements" {
            meets_requirements = true;
            implements_interfaces(p, false);
        } else {
            p.push_err(create_err!(
                p.peek_data()
                    .unwrap_or_else(|| String::from("no further data")),
                "Unexpected Name in Object Type Definition, {}",
                p.peek_data()
                    .unwrap_or_else(|| String::from("no further data")),
            ));
        }
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
        p.push_err(
            create_err!(
                p
                    .peek_data()
                    .unwrap_or_else(|| String::from("no further data")),
                "Expected Object Type Extension to have an Implements Interface, Directives, or Fields definition, got {}",
                p
                    .peek_data()
                    .unwrap_or_else(|| String::from("no further data")),
            ));
    }
}

/// See: https://spec.graphql.org/June2018/#ImplementsInterfaces
///
/// ```txt
/// ImplementsInterfaces
///     implements &[opt] NamedType
///     ImplementsInterfaces & NamedType
/// ```
pub(crate) fn implements_interfaces(p: &mut Parser, is_interfaces: bool) {
    let _guard = p.start_node(SyntaxKind::IMPLEMENTS_INTERFACES);
    p.bump(SyntaxKind::implements_KW);

    match p.peek() {
        Some(TokenKind::Name) => {
            let node = p.peek_data().unwrap();
            match node.as_str() {
                "&" => {
                    p.bump(SyntaxKind::AMP);
                    implements_interfaces(p, is_interfaces)
                }
                _ => name::name(p),
            }
        }
        _ => {
            if !is_interfaces {
                p.push_err(create_err!(
                    p.peek_data()
                        .unwrap_or_else(|| String::from("no further data")),
                    "Expected to have Implements Interfaces in a Object Type Definition, got {}",
                    p.peek_data()
                        .unwrap_or_else(|| String::from("no further data"))
                ));
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::parser::utils;

    #[test]
    fn it_parses_object_type_definition() {
        utils::check_ast(
            "
            type Person implements Human {
              name: String
              age: Int
              picture: Url
            }",
            r#"
            - DOCUMENT@0..44
                - OBJECT_TYPE_DEFINITION@0..44
                    - type_KW@0..4 "type"
                    - NAME@4..10
                        - IDENT@4..10 "Person"
                    - IMPLEMENTS_INTERFACES@10..25
                        - implements_KW@10..20 "implements"
                        - NAME@20..25
                            - IDENT@20..25 "Human"
                    - FIELDS_DEFINITION@25..44
                        - L_CURLY@25..26 "{"
                        - FIELD_DEFINITION@26..31
                            - NAME@26..30
                                - IDENT@26..30 "name"
                            - COLON@30..31 ":"
                            - TYPE@31..31
                                - NAMED_TYPE@31..31
                        - FIELD_DEFINITION@31..35
                            - NAME@31..34
                                - IDENT@31..34 "age"
                            - COLON@34..35 ":"
                            - TYPE@35..35
                                - NAMED_TYPE@35..35
                        - FIELD_DEFINITION@35..43
                            - NAME@35..42
                                - IDENT@35..42 "picture"
                            - COLON@42..43 ":"
                            - TYPE@43..43
                                - NAMED_TYPE@43..43
                        - R_CURLY@43..44 "}"
            "#,
        )
    }

    #[test]
    fn it_parses_extension() {
        utils::check_ast(
            "
            extend type Person implements Human @deprecated {
              name: String
              age: Int
              picture: Url
            }",
            r#"
            - DOCUMENT@0..61
                - OBJECT_TYPE_EXTENSION@0..61
                    - extend_KW@0..6 "extend"
                    - type_KW@6..10 "type"
                    - NAME@10..16
                        - IDENT@10..16 "Person"
                    - IMPLEMENTS_INTERFACES@16..31
                        - implements_KW@16..26 "implements"
                        - NAME@26..31
                            - IDENT@26..31 "Human"
                    - DIRECTIVES@31..42
                        - DIRECTIVE@31..42
                            - AT@31..32 "@"
                            - NAME@32..42
                                - IDENT@32..42 "deprecated"
                    - FIELDS_DEFINITION@42..61
                        - L_CURLY@42..43 "{"
                        - FIELD_DEFINITION@43..48
                            - NAME@43..47
                                - IDENT@43..47 "name"
                            - COLON@47..48 ":"
                            - TYPE@48..48
                                - NAMED_TYPE@48..48
                        - FIELD_DEFINITION@48..52
                            - NAME@48..51
                                - IDENT@48..51 "age"
                            - COLON@51..52 ":"
                            - TYPE@52..52
                                - NAMED_TYPE@52..52
                        - FIELD_DEFINITION@52..60
                            - NAME@52..59
                                - IDENT@52..59 "picture"
                            - COLON@59..60 ":"
                            - TYPE@60..60
                                - NAMED_TYPE@60..60
                        - R_CURLY@60..61 "}"
            "#,
        )
    }

    #[test]
    fn it_errors_when_extesion_name_is_missing() {
        utils::check_ast(
            "
            extend type {
              name: String
              age: Int
              picture: Url
            }",
            r#"
            - DOCUMENT@0..29
                - OBJECT_TYPE_EXTENSION@0..29
                    - extend_KW@0..6 "extend"
                    - type_KW@6..10 "type"
                    - FIELDS_DEFINITION@10..29
                        - L_CURLY@10..11 "{"
                        - FIELD_DEFINITION@11..16
                            - NAME@11..15
                                - IDENT@11..15 "name"
                            - COLON@15..16 ":"
                            - TYPE@16..16
                                - NAMED_TYPE@16..16
                        - FIELD_DEFINITION@16..20
                            - NAME@16..19
                                - IDENT@16..19 "age"
                            - COLON@19..20 ":"
                            - TYPE@20..20
                                - NAMED_TYPE@20..20
                        - FIELD_DEFINITION@20..28
                            - NAME@20..27
                                - IDENT@20..27 "picture"
                            - COLON@27..28 ":"
                            - TYPE@28..28
                                - NAMED_TYPE@28..28
                        - R_CURLY@28..29 "}"
            - ERROR@0:1 "Expected Object Type Extension to have a Name, got {"
            "#,
        )
    }

    #[test]
    fn it_errors_when_extesion_is_missing_required_syntax() {
        utils::check_ast(
            "extend type Person",
            r#"
            - DOCUMENT@0..16
                - OBJECT_TYPE_EXTENSION@0..16
                    - extend_KW@0..6 "extend"
                    - type_KW@6..10 "type"
                    - NAME@10..16
                        - IDENT@10..16 "Person"
            - ERROR@0:15 "Expected Object Type Extension to have an Implements Interface, Directives, or Fields definition, got no further data"
            "#,
        )
    }
}
