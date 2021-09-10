use crate::parser::grammar::{directive, name, value};
use crate::{create_err, Parser, SyntaxKind, TokenKind};

/// See: https://spec.graphql.org/June2018/#EnumTypeDefinition
///
/// ```txt
// EnumTypeDefinition
//     Description[opt] enum Name Directives[Const][opt] EnumValuesDefinition[opt]
/// ```
pub(crate) fn enum_type_definition(parser: &mut Parser) {
    let _guard = parser.start_node(SyntaxKind::ENUM_TYPE_DEFINITION);
    parser.bump(SyntaxKind::enum_KW);

    match parser.peek() {
        Some(TokenKind::Node) => name::name(parser),
        _ => {
            parser.push_err(create_err!(
                parser
                    .peek_data()
                    .unwrap_or_else(|| String::from("no further data")),
                "Expected Union Type Definition to have a Name, got {}",
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
        enum_values_definition(parser);
    }
}

/// See: https://spec.graphql.org/June2018/#EnumTypeExtension
///
/// ```txt
// EnumTypeExtension
///    extend enum Name Directives[Const][opt] EnumValuesDefinition
///    extend enum Name Directives[Const]
/// ```
pub(crate) fn enum_type_extension(parser: &mut Parser) {
    let _guard = parser.start_node(SyntaxKind::ENUM_TYPE_EXTENSION);
    parser.bump(SyntaxKind::extend_KW);
    parser.bump(SyntaxKind::enum_KW);

    let mut meets_requirements = false;

    match parser.peek() {
        Some(TokenKind::Node) => name::name(parser),
        _ => {
            parser.push_err(create_err!(
                parser
                    .peek_data()
                    .unwrap_or_else(|| String::from("no further data")),
                "Expected Union Type Extension to have a Name, got {}",
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
        enum_values_definition(parser);
    }

    if !meets_requirements {
        parser.push_err(create_err!(
            parser
                .peek_data()
                .unwrap_or_else(|| String::from("no further data")),
            "Expected Enum Type Extension to have Directives or Enum Values Definition, got {}",
            parser
                .peek_data()
                .unwrap_or_else(|| String::from("no further data")),
        ));
    }
}

/// See: https://spec.graphql.org/June2018/#EnumValuesDefinition
///
/// ```txt
/// EnumValuesDefinition
///     { EnumValueDefinition[list] }
/// ```
pub(crate) fn enum_values_definition(parser: &mut Parser) {
    let _guard = parser.start_node(SyntaxKind::ENUM_VALUES_DEFINITION);
    parser.bump(SyntaxKind::L_CURLY);

    match parser.peek() {
        Some(TokenKind::Node) => enum_value_definition(parser),
        _ => {
            parser.push_err(create_err!(
                parser
                    .peek_data()
                    .unwrap_or_else(|| String::from("no further data")),
                "Expected Enum Value Definition to follow, got {}",
                parser
                    .peek_data()
                    .unwrap_or_else(|| String::from("no further data"))
            ));
        }
    }

    if let Some(TokenKind::RCurly) = parser.peek() {
        parser.bump(SyntaxKind::R_CURLY)
    } else {
        parser.push_err(create_err!(
            parser
                .peek_data()
                .unwrap_or_else(|| String::from("no further data")),
            "Expected Enum Values Definition to have a closing }}, got {}",
            parser
                .peek_data()
                .unwrap_or_else(|| String::from("no further data"))
        ));
    }
}

/// See: https://spec.graphql.org/June2018/#EnumValueDefinition
///
/// ```txt
/// EnumValueDefinition
///     Description[opt] EnumValue Directives[Const][opt]
/// ```
pub(crate) fn enum_value_definition(parser: &mut Parser) {
    if let Some(TokenKind::Node) = parser.peek() {
        let guard = parser.start_node(SyntaxKind::ENUM_VALUE_DEFINITION);
        value::enum_value(parser);

        if let Some(TokenKind::At) = parser.peek() {
            directive::directives(parser);
        }
        if parser.peek().is_some() {
            guard.finish_node();
            return enum_value_definition(parser);
        }
    }

    if let Some(TokenKind::Comma) = parser.peek() {
        parser.bump(SyntaxKind::COMMA);
        return enum_value_definition(parser);
    }

    if let Some(TokenKind::RCurly) = parser.peek() {
        return;
    }
}

#[cfg(test)]
mod test {
    use crate::parser::utils;

    #[test]
    fn it_parses_enum_type_definition() {
        utils::check_ast(
            "enum Direction {
              NORTH
              EAST
              SOUTH
              WEST
            }",
            r#"
            - DOCUMENT@0..33
                - ENUM_TYPE_DEFINITION@0..33
                    - enum_KW@0..4 "enum"
                    - NAME@4..13
                        - IDENT@4..13 "Direction"
                    - ENUM_VALUES_DEFINITION@13..33
                        - L_CURLY@13..14 "{"
                        - ENUM_VALUE_DEFINITION@14..19
                            - ENUM_VALUE@14..19
                                - NAME@14..19
                                    - IDENT@14..19 "NORTH"
                        - ENUM_VALUE_DEFINITION@19..23
                            - ENUM_VALUE@19..23
                                - NAME@19..23
                                    - IDENT@19..23 "EAST"
                        - ENUM_VALUE_DEFINITION@23..28
                            - ENUM_VALUE@23..28
                                - NAME@23..28
                                    - IDENT@23..28 "SOUTH"
                        - ENUM_VALUE_DEFINITION@28..32
                            - ENUM_VALUE@28..32
                                - NAME@28..32
                                    - IDENT@28..32 "WEST"
                        - R_CURLY@32..33 "}"
            "#,
        )
    }

    #[test]
    fn it_creates_an_error_when_name_is_missing() {
        utils::check_ast(
            "enum {
              NORTH
              EAST
              SOUTH
              WEST
            }",
            r#"
            - DOCUMENT@0..24
                - ENUM_TYPE_DEFINITION@0..24
                    - enum_KW@0..4 "enum"
                    - ENUM_VALUES_DEFINITION@4..24
                        - L_CURLY@4..5 "{"
                        - ENUM_VALUE_DEFINITION@5..10
                            - ENUM_VALUE@5..10
                                - NAME@5..10
                                    - IDENT@5..10 "NORTH"
                        - ENUM_VALUE_DEFINITION@10..14
                            - ENUM_VALUE@10..14
                                - NAME@10..14
                                    - IDENT@10..14 "EAST"
                        - ENUM_VALUE_DEFINITION@14..19
                            - ENUM_VALUE@14..19
                                - NAME@14..19
                                    - IDENT@14..19 "SOUTH"
                        - ENUM_VALUE_DEFINITION@19..23
                            - ENUM_VALUE@19..23
                                - NAME@19..23
                                    - IDENT@19..23 "WEST"
                        - R_CURLY@23..24 "}"
            - ERROR@0:1 "Expected Union Type Definition to have a Name, got {"
            "#,
        )
    }

    #[test]
    fn it_creates_an_error_when_enum_values_are_missing() {
        utils::check_ast(
            "enum Direction {
            }",
            r#"
            - DOCUMENT@0..15
                - ENUM_TYPE_DEFINITION@0..15
                    - enum_KW@0..4 "enum"
                    - NAME@4..13
                        - IDENT@4..13 "Direction"
                    - ENUM_VALUES_DEFINITION@13..15
                        - L_CURLY@13..14 "{"
                        - R_CURLY@14..15 "}"
            - ERROR@0:1 "Expected Enum Value Definition to follow, got }"
            "#,
        )
    }

    #[test]
    fn it_parses_extension() {
        utils::check_ast(
            "extend enum Direction @deprecated {
              SOUTH
              WEST
            }",
            r#"
            - DOCUMENT@0..41
                - ENUM_TYPE_EXTENSION@0..41
                    - extend_KW@0..6 "extend"
                    - enum_KW@6..10 "enum"
                    - NAME@10..19
                        - IDENT@10..19 "Direction"
                    - DIRECTIVES@19..30
                        - DIRECTIVE@19..30
                            - AT@19..20 "@"
                            - NAME@20..30
                                - IDENT@20..30 "deprecated"
                    - ENUM_VALUES_DEFINITION@30..41
                        - L_CURLY@30..31 "{"
                        - ENUM_VALUE_DEFINITION@31..36
                            - ENUM_VALUE@31..36
                                - NAME@31..36
                                    - IDENT@31..36 "SOUTH"
                        - ENUM_VALUE_DEFINITION@36..40
                            - ENUM_VALUE@36..40
                                - NAME@36..40
                                    - IDENT@36..40 "WEST"
                        - R_CURLY@40..41 "}"
            "#,
        )
    }

    #[test]
    fn it_creates_an_error_when_name_is_missing_in_extension() {
        utils::check_ast(
            "extend enum {
              NORTH
              EAST
            }",
            r#"
            - DOCUMENT@0..21
                - ENUM_TYPE_EXTENSION@0..21
                    - extend_KW@0..6 "extend"
                    - enum_KW@6..10 "enum"
                    - ENUM_VALUES_DEFINITION@10..21
                        - L_CURLY@10..11 "{"
                        - ENUM_VALUE_DEFINITION@11..16
                            - ENUM_VALUE@11..16
                                - NAME@11..16
                                    - IDENT@11..16 "NORTH"
                        - ENUM_VALUE_DEFINITION@16..20
                            - ENUM_VALUE@16..20
                                - NAME@16..20
                                    - IDENT@16..20 "EAST"
                        - R_CURLY@20..21 "}"
            - ERROR@0:1 "Expected Union Type Extension to have a Name, got {"
            "#,
        )
    }

    #[test]
    fn it_creates_an_error_when_requirements_are_missing_in_extension() {
        utils::check_ast(
            "extend enum Direction",
            r#"
            - DOCUMENT@0..19
                - ENUM_TYPE_EXTENSION@0..19
                    - extend_KW@0..6 "extend"
                    - enum_KW@6..10 "enum"
                    - NAME@10..19
                        - IDENT@10..19 "Direction"
            - ERROR@0:15 "Expected Enum Type Extension to have Directives or Enum Values Definition, got no further data"
            "#,
        )
    }
}
