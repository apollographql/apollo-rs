use crate::parser::grammar::{directive, field, name};
use crate::{Parser, SyntaxKind, TokenKind, T};

/// See: https://spec.graphql.org/June2018/#InterfaceTypeDefinition
///
/// ```txt
/// InterfaceTypeDefinition
///     Description[opt] interface Name Directives[Const][opt] FieldsDefinition[opt]
/// ```
pub(crate) fn interface_type_definition(p: &mut Parser) {
    let _guard = p.start_node(SyntaxKind::INTERFACE_TYPE_DEFINITION);
    p.bump(SyntaxKind::interface_KW);

    match p.peek() {
        Some(TokenKind::Name) => name::name(p),
        _ => p.err("expected a Name"),
    }

    if let Some(T![@]) = p.peek() {
        directive::directives(p);
    }

    if let Some(T!['{']) = p.peek() {
        field::fields_definition(p);
    }
}

/// See: https://spec.graphql.org/June2018/#InterfaceTypeExtension
///
/// ```txt
/// InterfaceTypeExtension
///     extend interface Name Directives[Const][opt] FieldsDefinition
///     extend interface Name Directives[Const]
/// ```
pub(crate) fn interface_type_extension(p: &mut Parser) {
    let _guard = p.start_node(SyntaxKind::INTERFACE_TYPE_EXTENSION);
    p.bump(SyntaxKind::extend_KW);
    p.bump(SyntaxKind::interface_KW);

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
        field::fields_definition(p);
    }

    if !meets_requirements {
        p.err("exptected Directives or a Fields Definition");
    }
}

#[cfg(test)]
mod test {
    use crate::parser::utils;

    #[test]
    fn it_parses_definition() {
        utils::check_ast(
            "
            interface ValuedEntity {
              value: Int
            }",
            r#"
            - DOCUMENT@0..29
                - INTERFACE_TYPE_DEFINITION@0..29
                    - interface_KW@0..9 "interface"
                    - NAME@9..21
                        - IDENT@9..21 "ValuedEntity"
                    - FIELDS_DEFINITION@21..29
                        - L_CURLY@21..22 "{"
                        - FIELD_DEFINITION@22..28
                            - NAME@22..27
                                - IDENT@22..27 "value"
                            - COLON@27..28 ":"
                            - TYPE@28..28
                                - NAMED_TYPE@28..28
                        - R_CURLY@28..29 "}"
            "#,
        )
    }

    #[test]
    fn it_parses_extension() {
        utils::check_ast(
            "
            extend interface ValuedEntity @skip {
              value: Int
            }",
            r#"
            - DOCUMENT@0..40
                - INTERFACE_TYPE_EXTENSION@0..40
                    - extend_KW@0..6 "extend"
                    - interface_KW@6..15 "interface"
                    - NAME@15..27
                        - IDENT@15..27 "ValuedEntity"
                    - DIRECTIVES@27..32
                        - DIRECTIVE@27..32
                            - AT@27..28 "@"
                            - NAME@28..32
                                - IDENT@28..32 "skip"
                    - FIELDS_DEFINITION@32..40
                        - L_CURLY@32..33 "{"
                        - FIELD_DEFINITION@33..39
                            - NAME@33..38
                                - IDENT@33..38 "value"
                            - COLON@38..39 ":"
                            - TYPE@39..39
                                - NAMED_TYPE@39..39
                        - R_CURLY@39..40 "}"
            "#,
        )
    }

    #[test]
    fn it_errors_when_extension_is_missing_name() {
        utils::check_ast(
            "
            extend interface {
              value: Int
            }",
            r#"
            - DOCUMENT@0..23
                - INTERFACE_TYPE_EXTENSION@0..23
                    - extend_KW@0..6 "extend"
                    - interface_KW@6..15 "interface"
                    - FIELDS_DEFINITION@15..23
                        - L_CURLY@15..16 "{"
                        - FIELD_DEFINITION@16..22
                            - NAME@16..21
                                - IDENT@16..21 "value"
                            - COLON@21..22 ":"
                            - TYPE@22..22
                                - NAMED_TYPE@22..22
                        - R_CURLY@22..23 "}"
            - ERROR@0:1 "expected a Name"
            "#,
        )
    }

    #[test]
    fn it_errors_when_required_syntax_is_missing() {
        utils::check_ast(
            "extend interface ValuedEntity",
            r#"
            - DOCUMENT@0..27
                - INTERFACE_TYPE_EXTENSION@0..27
                    - extend_KW@0..6 "extend"
                    - interface_KW@6..15 "interface"
                    - NAME@15..27
                        - IDENT@15..27 "ValuedEntity"
            - ERROR@0:3 "exptected Directives or a Fields Definition"
            "#,
        )
    }
}
