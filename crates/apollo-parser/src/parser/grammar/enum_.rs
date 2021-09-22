use crate::parser::grammar::{directive, name, value};
use crate::{Parser, SyntaxKind, TokenKind, S, T};

/// See: https://spec.graphql.org/June2018/#EnumTypeDefinition
///
/// ```txt
// EnumTypeDefinition
//     Description[opt] enum Name Directives[Const][opt] EnumValuesDefinition[opt]
/// ```
pub(crate) fn enum_type_definition(p: &mut Parser) {
    let _g = p.start_node(SyntaxKind::ENUM_TYPE_DEFINITION);
    p.bump(SyntaxKind::enum_KW);

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

/// See: https://spec.graphql.org/June2018/#EnumTypeExtension
///
/// ```txt
// EnumTypeExtension
///    extend enum Name Directives[Const][opt] EnumValuesDefinition
///    extend enum Name Directives[Const]
/// ```
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

/// See: https://spec.graphql.org/June2018/#EnumValuesDefinition
///
/// ```txt
/// EnumValuesDefinition
///     { EnumValueDefinition[list] }
/// ```
pub(crate) fn enum_values_definition(p: &mut Parser) {
    let g = p.start_node(SyntaxKind::ENUM_VALUES_DEFINITION);
    p.bump(S!['{']);

    match p.peek() {
        Some(TokenKind::Name) => enum_value_definition(p),
        _ => p.err("expected Enum Value Definition"),
    }

    p.expect(T!['}'], S!['}']);
    g.finish_node();
    p.bump_ignored();
}

/// See: https://spec.graphql.org/June2018/#EnumValueDefinition
///
/// ```txt
/// EnumValueDefinition
///     Description[opt] EnumValue Directives[Const][opt]
/// ```
pub(crate) fn enum_value_definition(p: &mut Parser) {
    if let Some(TokenKind::Name) = p.peek() {
        let guard = p.start_node(SyntaxKind::ENUM_VALUE_DEFINITION);
        value::enum_value(p);

        if let Some(T![@]) = p.peek() {
            directive::directives(p);
        }
        if p.peek().is_some() {
            guard.finish_node();
            return enum_value_definition(p);
        }
    }

    if let Some(T![,]) = p.peek() {
        p.bump(S![,]);
        return enum_value_definition(p);
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
              NORTH
              EAST
              SOUTH
              WEST
            }",
            r#"
            - DOCUMENT@0..108
                - ENUM_TYPE_DEFINITION@0..108
                    - enum_KW@0..4 "enum"
                    - WHITESPACE@4..5 " "
                    - NAME@5..15
                        - IDENT@5..14 "Direction"
                        - WHITESPACE@14..15 " "
                    - ENUM_VALUES_DEFINITION@15..108
                        - L_CURLY@15..16 "{"
                        - WHITESPACE@16..31 "\n              "
                        - ENUM_VALUE_DEFINITION@31..51
                            - ENUM_VALUE@31..51
                                - NAME@31..51
                                    - IDENT@31..36 "NORTH"
                                    - WHITESPACE@36..51 "\n              "
                        - ENUM_VALUE_DEFINITION@51..70
                            - ENUM_VALUE@51..70
                                - NAME@51..70
                                    - IDENT@51..55 "EAST"
                                    - WHITESPACE@55..70 "\n              "
                        - ENUM_VALUE_DEFINITION@70..90
                            - ENUM_VALUE@70..90
                                - NAME@70..90
                                    - IDENT@70..75 "SOUTH"
                                    - WHITESPACE@75..90 "\n              "
                        - ENUM_VALUE_DEFINITION@90..107
                            - ENUM_VALUE@90..107
                                - NAME@90..107
                                    - IDENT@90..94 "WEST"
                                    - WHITESPACE@94..107 "\n            "
                        - R_CURLY@107..108 "}"
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
            - ERROR@0:1 "expected a Name"
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
            - ERROR@0:1 "expected Enum Value Definition"
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
            - ERROR@0:3 "expected R_CURLY, got EOF"
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
            - ERROR@0:1 "expected a Name"
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
            - ERROR@0:3 "expected Directived or Enum Values Definition"
            "#,
        )
    }
}
