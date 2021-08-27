use crate::parser::{directive, field, name};
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

#[cfg(test)]
mod test {
    use crate::parser::utils;

    #[test]
    fn it_parses_interface_type_definition() {
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
                        - L_CURLY@28..29 "}"
            "#,
        )
    }
}
