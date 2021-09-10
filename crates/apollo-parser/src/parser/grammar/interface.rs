use crate::parser::grammar::{directive, field, name};
use crate::{create_err, Parser, SyntaxKind, TokenKind};

/// See: https://spec.graphql.org/June2018/#InterfaceTypeDefinition
///
/// ```txt
/// InterfaceTypeDefinition
///     Description[opt] interface Name Directives[Const][opt] FieldsDefinition[opt]
/// ```
pub(crate) fn interface_type_definition(parser: &mut Parser) {
    let _guard = parser.start_node(SyntaxKind::INTERFACE_TYPE_DEFINITION);
    parser.bump(SyntaxKind::interface_KW);

    match parser.peek() {
        Some(TokenKind::Node) => name::name(parser),
        _ => {
            parser.push_err(create_err!(
                parser
                    .peek_data()
                    .unwrap_or_else(|| String::from("no further data")),
                "Expected Interface Type Definition to have a Name, got {}",
                parser
                    .peek_data()
                    .unwrap_or_else(|| String::from("no further data")),
            ));
        }
    }

    if let Some(TokenKind::At) = parser.peek() {
        directive::directives(parser);
    }

    if let Some(TokenKind::LCurly) = parser.peek() {
        field::fields_definition(parser);
    }
}

/// See: https://spec.graphql.org/June2018/#InterfaceTypeExtension
///
/// ```txt
/// InterfaceTypeExtension
///     extend interface Name Directives[Const][opt] FieldsDefinition
///     extend interface Name Directives[Const]
/// ```
pub(crate) fn interface_type_extension(parser: &mut Parser) {
    let _guard = parser.start_node(SyntaxKind::INTERFACE_TYPE_EXTENSION);
    parser.bump(SyntaxKind::extend_KW);
    parser.bump(SyntaxKind::interface_KW);

    let mut meets_requirements = false;

    match parser.peek() {
        Some(TokenKind::Node) => name::name(parser),
        _ => {
            parser.push_err(create_err!(
                parser
                    .peek_data()
                    .unwrap_or_else(|| String::from("no further data")),
                "Expected Interface Type Definition to have a Name, got {}",
                parser
                    .peek_data()
                    .unwrap_or_else(|| String::from("no further data")),
            ));
        }
    }

    if let Some(TokenKind::At) = parser.peek() {
        meets_requirements = true;
        directive::directives(parser);
    }

    if let Some(TokenKind::LCurly) = parser.peek() {
        meets_requirements = true;
        field::fields_definition(parser);
    }

    if !meets_requirements {
        parser.push_err(create_err!(
            parser
                .peek_data()
                .unwrap_or_else(|| String::from("no further data")),
            "Expected Interface Type Extension to have a Directives or Fields definition, got {}",
            parser
                .peek_data()
                .unwrap_or_else(|| String::from("no further data")),
        ));
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
            - ERROR@0:1 "Expected Interface Type Definition to have a Name, got {"
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
            - ERROR@0:15 "Expected Interface Type Extension to have a Directives or Fields definition, got no further data"
            "#,
        )
    }
}
