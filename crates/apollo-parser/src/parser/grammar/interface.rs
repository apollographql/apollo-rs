use crate::parser::grammar::{directive, field, name};
use crate::{Parser, SyntaxKind, TokenKind, T};

/// See: https://spec.graphql.org/June2018/#InterfaceTypeDefinition
///
/// ```txt
/// InterfaceTypeDefinition
///     Description[opt] interface Name Directives[Const][opt] FieldsDefinition[opt]
/// ```
pub(crate) fn interface_type_definition(p: &mut Parser) {
    let _g = p.start_node(SyntaxKind::INTERFACE_TYPE_DEFINITION);
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
    let _g = p.start_node(SyntaxKind::INTERFACE_TYPE_EXTENSION);
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
            - DOCUMENT@0..76
                - WHITESPACE@0..13 "\n            "
                - INTERFACE_TYPE_DEFINITION@13..76
                    - interface_KW@13..22 "interface"
                    - WHITESPACE@22..23 " "
                    - NAME@23..36
                        - IDENT@23..35 "ValuedEntity"
                        - WHITESPACE@35..36 " "
                    - FIELDS_DEFINITION@36..76
                        - L_CURLY@36..37 "{"
                        - WHITESPACE@37..52 "\n              "
                        - FIELD_DEFINITION@52..75
                            - NAME@52..57
                                - IDENT@52..57 "value"
                            - COLON@57..58 ":"
                            - WHITESPACE@58..59 " "
                            - TYPE@59..75
                                - WHITESPACE@59..72 "\n            "
                                - NAMED_TYPE@72..75
                                    - NAME@72..75
                                        - IDENT@72..75 "Int"
                        - R_CURLY@75..76 "}"
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
            - DOCUMENT@0..89
                - WHITESPACE@0..13 "\n            "
                - INTERFACE_TYPE_EXTENSION@13..89
                    - extend_KW@13..19 "extend"
                    - WHITESPACE@19..20 " "
                    - interface_KW@20..29 "interface"
                    - WHITESPACE@29..30 " "
                    - NAME@30..43
                        - IDENT@30..42 "ValuedEntity"
                        - WHITESPACE@42..43 " "
                    - DIRECTIVES@43..49
                        - DIRECTIVE@43..49
                            - AT@43..44 "@"
                            - NAME@44..49
                                - IDENT@44..48 "skip"
                                - WHITESPACE@48..49 " "
                    - FIELDS_DEFINITION@49..89
                        - L_CURLY@49..50 "{"
                        - WHITESPACE@50..65 "\n              "
                        - FIELD_DEFINITION@65..88
                            - NAME@65..70
                                - IDENT@65..70 "value"
                            - COLON@70..71 ":"
                            - WHITESPACE@71..72 " "
                            - TYPE@72..88
                                - WHITESPACE@72..85 "\n            "
                                - NAMED_TYPE@85..88
                                    - NAME@85..88
                                        - IDENT@85..88 "Int"
                        - R_CURLY@88..89 "}"
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
            - DOCUMENT@0..70
                - WHITESPACE@0..13 "\n            "
                - INTERFACE_TYPE_EXTENSION@13..70
                    - extend_KW@13..19 "extend"
                    - WHITESPACE@19..20 " "
                    - interface_KW@20..29 "interface"
                    - WHITESPACE@29..30 " "
                    - FIELDS_DEFINITION@30..70
                        - L_CURLY@30..31 "{"
                        - WHITESPACE@31..46 "\n              "
                        - FIELD_DEFINITION@46..69
                            - NAME@46..51
                                - IDENT@46..51 "value"
                            - COLON@51..52 ":"
                            - WHITESPACE@52..53 " "
                            - TYPE@53..69
                                - WHITESPACE@53..66 "\n            "
                                - NAMED_TYPE@66..69
                                    - NAME@66..69
                                        - IDENT@66..69 "Int"
                        - R_CURLY@69..70 "}"
            - ERROR@0:1 "expected a Name"
            "#,
        )
    }

    #[test]
    fn it_errors_when_required_syntax_is_missing() {
        utils::check_ast(
            "extend interface ValuedEntity",
            r#"
            - DOCUMENT@0..29
                - INTERFACE_TYPE_EXTENSION@0..29
                    - extend_KW@0..6 "extend"
                    - WHITESPACE@6..7 " "
                    - interface_KW@7..16 "interface"
                    - WHITESPACE@16..17 " "
                    - NAME@17..29
                        - IDENT@17..29 "ValuedEntity"
            - ERROR@0:3 "exptected Directives or a Fields Definition"
            "#,
        )
    }
}
