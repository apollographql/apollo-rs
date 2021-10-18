use crate::{
    parser::grammar::{description, directive, name, value},
    Parser, SyntaxKind, TokenKind, S, T,
};

/// See: https://spec.graphql.org/draft/#EnumTypeDefinition
///
/// *EnumTypeDefinition*:
///     Description<sub>opt</sub> **enum** Name Directives<sub>\[Const\] opt</sub> EnumValuesDefinition <sub>opt</sub>
pub(crate) fn enum_type_definition(p: &mut Parser) {
    let _g = p.start_node(SyntaxKind::ENUM_TYPE_DEFINITION);

    if let Some(TokenKind::StringValue) = p.peek() {
        description::description(p);
    }

    if let Some("enum") = p.peek_data().as_deref() {
        p.bump(SyntaxKind::enum_KW);
    }

    match p.peek() {
        Some(TokenKind::Name) => name::name(p),
        _ => p.err("expected a Name"),
    }

    if let Some(T![@]) = p.peek() {
        directive::directives(p);
    }

    if let Some(T!['{']) = p.peek() {
        enum_values_definition(p);
    }
}

/// See: https://spec.graphql.org/draft/#EnumTypeExtension
///
// *EnumTypeExtension*:
///    **extend** **enum** Name Directives<sub>\[Const\] opt</sub> EnumValuesDefinition
///    **extend** **enum** Name Directives<sub>\[Const\]</sub>
pub(crate) fn enum_type_extension(p: &mut Parser) {
    let _g = p.start_node(SyntaxKind::ENUM_TYPE_EXTENSION);
    p.bump(SyntaxKind::extend_KW);
    p.bump(SyntaxKind::enum_KW);

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
        enum_values_definition(p);
    }

    if !meets_requirements {
        p.err("expected Directived or Enum Values Definition");
    }
}

/// See: https://spec.graphql.org/draft/#EnumValuesDefinition
///
/// *EnumValuesDefinition*:
///     **{** EnumValueDefinition<sub>list</sub> **}**
pub(crate) fn enum_values_definition(p: &mut Parser) {
    let _g = p.start_node(SyntaxKind::ENUM_VALUES_DEFINITION);
    p.bump(S!['{']);

    match p.peek() {
        Some(TokenKind::Name | TokenKind::StringValue) => enum_value_definition(p),
        _ => p.err("expected Enum Value Definition"),
    }

    p.expect(T!['}'], S!['}']);
}

/// See: https://spec.graphql.org/draft/#EnumValueDefinition
///
/// *EnumValueDefinition*:
///     Description<sub>opt</sub> EnumValue Directives<sub>\[Const\] opt</sub>
pub(crate) fn enum_value_definition(p: &mut Parser) {
    if let Some(TokenKind::Name | TokenKind::StringValue) = p.peek() {
        let guard = p.start_node(SyntaxKind::ENUM_VALUE_DEFINITION);

        if let Some(TokenKind::StringValue) = p.peek() {
            description::description(p);
        }

        value::enum_value(p);

        if let Some(T![@]) = p.peek() {
            directive::directives(p);
        }
        if p.peek().is_some() {
            guard.finish_node();
            return enum_value_definition(p);
        }
    }

    if let Some(T!['}']) = p.peek() {
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
              \"\"\"
              enum description
              \"\"\"
              NORTH
              EAST
              SOUTH
              WEST
            }",
            r#"
            - DOCUMENT@0..174
                - ENUM_TYPE_DEFINITION@0..174
                    - enum_KW@0..4 "enum"
                    - WHITESPACE@4..5 " "
                    - NAME@5..15
                        - IDENT@5..14 "Direction"
                        - WHITESPACE@14..15 " "
                    - ENUM_VALUES_DEFINITION@15..174
                        - L_CURLY@15..16 "{"
                        - WHITESPACE@16..31 "\n              "
                        - ENUM_VALUE_DEFINITION@31..117
                            - DESCRIPTION@31..97
                                - STRING_VALUE@31..82 "\"\"\n              enum description\n              \"\"\""
                                - WHITESPACE@82..97 "\n              "
                            - ENUM_VALUE@97..117
                                - NAME@97..117
                                    - IDENT@97..102 "NORTH"
                                    - WHITESPACE@102..117 "\n              "
                        - ENUM_VALUE_DEFINITION@117..136
                            - ENUM_VALUE@117..136
                                - NAME@117..136
                                    - IDENT@117..121 "EAST"
                                    - WHITESPACE@121..136 "\n              "
                        - ENUM_VALUE_DEFINITION@136..156
                            - ENUM_VALUE@136..156
                                - NAME@136..156
                                    - IDENT@136..141 "SOUTH"
                                    - WHITESPACE@141..156 "\n              "
                        - ENUM_VALUE_DEFINITION@156..173
                            - ENUM_VALUE@156..173
                                - NAME@156..173
                                    - IDENT@156..160 "WEST"
                                    - WHITESPACE@160..173 "\n            "
                        - R_CURLY@173..174 "}"
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
            - DOCUMENT@0..98
                - ENUM_TYPE_DEFINITION@0..98
                    - enum_KW@0..4 "enum"
                    - WHITESPACE@4..5 " "
                    - ENUM_VALUES_DEFINITION@5..98
                        - L_CURLY@5..6 "{"
                        - WHITESPACE@6..21 "\n              "
                        - ENUM_VALUE_DEFINITION@21..41
                            - ENUM_VALUE@21..41
                                - NAME@21..41
                                    - IDENT@21..26 "NORTH"
                                    - WHITESPACE@26..41 "\n              "
                        - ENUM_VALUE_DEFINITION@41..60
                            - ENUM_VALUE@41..60
                                - NAME@41..60
                                    - IDENT@41..45 "EAST"
                                    - WHITESPACE@45..60 "\n              "
                        - ENUM_VALUE_DEFINITION@60..80
                            - ENUM_VALUE@60..80
                                - NAME@60..80
                                    - IDENT@60..65 "SOUTH"
                                    - WHITESPACE@65..80 "\n              "
                        - ENUM_VALUE_DEFINITION@80..97
                            - ENUM_VALUE@80..97
                                - NAME@80..97
                                    - IDENT@80..84 "WEST"
                                    - WHITESPACE@84..97 "\n            "
                        - R_CURLY@97..98 "}"
            - ERROR@5:6 "expected a Name"
            "#,
        )
    }

    #[test]
    fn it_creates_an_error_when_enum_values_are_missing() {
        utils::check_ast(
            "enum Direction {
            }",
            r#"
            - DOCUMENT@0..30
                - ENUM_TYPE_DEFINITION@0..30
                    - enum_KW@0..4 "enum"
                    - WHITESPACE@4..5 " "
                    - NAME@5..15
                        - IDENT@5..14 "Direction"
                        - WHITESPACE@14..15 " "
                    - ENUM_VALUES_DEFINITION@15..30
                        - L_CURLY@15..16 "{"
                        - WHITESPACE@16..29 "\n            "
                        - R_CURLY@29..30 "}"
            - ERROR@29:30 "expected Enum Value Definition"
            "#,
        )
    }

    #[test]
    fn it_creates_an_error_when_l_curly_is_missing() {
        utils::check_ast(
            "enum Direction { NORTH WEST",
            r#"
            - DOCUMENT@0..27
                - ENUM_TYPE_DEFINITION@0..27
                    - enum_KW@0..4 "enum"
                    - WHITESPACE@4..5 " "
                    - NAME@5..15
                        - IDENT@5..14 "Direction"
                        - WHITESPACE@14..15 " "
                    - ENUM_VALUES_DEFINITION@15..27
                        - L_CURLY@15..16 "{"
                        - WHITESPACE@16..17 " "
                        - ENUM_VALUE_DEFINITION@17..23
                            - ENUM_VALUE@17..23
                                - NAME@17..23
                                    - IDENT@17..22 "NORTH"
                                    - WHITESPACE@22..23 " "
                        - ENUM_VALUE_DEFINITION@23..27
                            - ENUM_VALUE@23..27
                                - NAME@23..27
                                    - IDENT@23..27 "WEST"
            - ERROR@27:27 "expected R_CURLY, got EOF"
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
            - DOCUMENT@0..88
                - ENUM_TYPE_EXTENSION@0..88
                    - extend_KW@0..6 "extend"
                    - WHITESPACE@6..7 " "
                    - enum_KW@7..11 "enum"
                    - WHITESPACE@11..12 " "
                    - NAME@12..22
                        - IDENT@12..21 "Direction"
                        - WHITESPACE@21..22 " "
                    - DIRECTIVES@22..34
                        - DIRECTIVE@22..34
                            - AT@22..23 "@"
                            - NAME@23..34
                                - IDENT@23..33 "deprecated"
                                - WHITESPACE@33..34 " "
                    - ENUM_VALUES_DEFINITION@34..88
                        - L_CURLY@34..35 "{"
                        - WHITESPACE@35..50 "\n              "
                        - ENUM_VALUE_DEFINITION@50..70
                            - ENUM_VALUE@50..70
                                - NAME@50..70
                                    - IDENT@50..55 "SOUTH"
                                    - WHITESPACE@55..70 "\n              "
                        - ENUM_VALUE_DEFINITION@70..87
                            - ENUM_VALUE@70..87
                                - NAME@70..87
                                    - IDENT@70..74 "WEST"
                                    - WHITESPACE@74..87 "\n            "
                        - R_CURLY@87..88 "}"
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
            - DOCUMENT@0..66
                - ENUM_TYPE_EXTENSION@0..66
                    - extend_KW@0..6 "extend"
                    - WHITESPACE@6..7 " "
                    - enum_KW@7..11 "enum"
                    - WHITESPACE@11..12 " "
                    - ENUM_VALUES_DEFINITION@12..66
                        - L_CURLY@12..13 "{"
                        - WHITESPACE@13..28 "\n              "
                        - ENUM_VALUE_DEFINITION@28..48
                            - ENUM_VALUE@28..48
                                - NAME@28..48
                                    - IDENT@28..33 "NORTH"
                                    - WHITESPACE@33..48 "\n              "
                        - ENUM_VALUE_DEFINITION@48..65
                            - ENUM_VALUE@48..65
                                - NAME@48..65
                                    - IDENT@48..52 "EAST"
                                    - WHITESPACE@52..65 "\n            "
                        - R_CURLY@65..66 "}"
            - ERROR@12:13 "expected a Name"
            "#,
        )
    }

    #[test]
    fn it_creates_an_error_when_requirements_are_missing_in_extension() {
        utils::check_ast(
            "extend enum Direction",
            r#"
            - DOCUMENT@0..21
                - ENUM_TYPE_EXTENSION@0..21
                    - extend_KW@0..6 "extend"
                    - WHITESPACE@6..7 " "
                    - enum_KW@7..11 "enum"
                    - WHITESPACE@11..12 " "
                    - NAME@12..21
                        - IDENT@12..21 "Direction"
            - ERROR@21:21 "expected Directived or Enum Values Definition"
            "#,
        )
    }
}
