use crate::parser::grammar::{description, directive, name, ty, value};
use crate::{Parser, SyntaxKind, TokenKind, S, T};

/// See: https://spec.graphql.org/draft/#InputObjectTypeDefinition
///
/// *InputObjectTypeDefinition*:
///     Description<sub>opt</sub> **input** Name Directives<sub>\[Const\] opt</sub> InputFieldsDefinition<sub>opt</sub>
/// ```
pub(crate) fn input_object_type_definition(p: &mut Parser) {
    let _g = p.start_node(SyntaxKind::INPUT_OBJECT_TYPE_DEFINITION);

    if let Some(TokenKind::StringValue) = p.peek() {
        description::description(p);
    }

    if let Some("input") = p.peek_data().as_deref() {
        p.bump(SyntaxKind::input_KW);
    }

    match p.peek() {
        Some(TokenKind::Name) => name::name(p),
        _ => p.err("expected a Name"),
    }

    if let Some(T![@]) = p.peek() {
        directive::directives(p);
    }

    if let Some(T!['{']) = p.peek() {
        input_fields_definition(p);
    }
}

/// See: https://spec.graphql.org/draft/#InputObjectTypeExtension
///
/// *InputObjectTypeExtension*:
///     **extend** **input** Name Directives<sub>\[Const\] opt</sub> InputFieldsDefinition
///     **extend** **input** Name Directives<sub>\[Const\]</sub>
pub(crate) fn input_object_type_extension(p: &mut Parser) {
    let _g = p.start_node(SyntaxKind::INPUT_OBJECT_TYPE_EXTENSION);
    p.bump(SyntaxKind::extend_KW);
    p.bump(SyntaxKind::input_KW);

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
        input_fields_definition(p);
    }

    if !meets_requirements {
        p.err("expected Directives or an Input Fields Definition");
    }
}

/// See: https://spec.graphql.org/draft/#InputFieldsDefinition
///
/// *InputFieldsDefinition*:
///     **{** InputValueDefinition<sub>list</sub> **}**
pub(crate) fn input_fields_definition(p: &mut Parser) {
    let _g = p.start_node(SyntaxKind::INPUT_FIELDS_DEFINITION);
    p.bump(S!['{']);
    input_value_definition(p, false);
    p.expect(T!['}'], S!['}']);
}

/// See: https://spec.graphql.org/draft/#InputValueDefinition
///
/// *InputValueDefinition*:
///     Description<sub>opt</sub> Name **:** Type DefaultValue<sub>opt</sub> Directives<sub>\[Const\] opt</sub>
pub(crate) fn input_value_definition(p: &mut Parser, is_input: bool) {
    if let Some(TokenKind::Name | TokenKind::StringValue) = p.peek() {
        let guard = p.start_node(SyntaxKind::INPUT_VALUE_DEFINITION);

        if let Some(TokenKind::StringValue) = p.peek() {
            description::description(p);
        }

        name::name(p);

        if let Some(T![:]) = p.peek() {
            p.bump(S![:]);
            match p.peek() {
                Some(TokenKind::Name) | Some(T!['[']) => {
                    ty::ty(p);
                    if let Some(T![=]) = p.peek() {
                        value::default_value(p);
                    }
                    if p.peek().is_some() {
                        guard.finish_node();
                        return input_value_definition(p, true);
                    }
                }
                _ => p.err("expected a Type"),
            }
        } else {
            p.err("expected a Name");
        }
    }
    // TODO @lrlna: this can be simplified a little bit, and follow the pattern of FieldDefinition
    if !is_input {
        p.err("expected an Input Value Definition");
    }
}

#[cfg(test)]
mod test {
    use crate::parser::utils;

    #[test]
    fn it_parses_definition() {
        utils::check_ast(
            "input ExampleInputObject {
              a: String
              b: Int!
            }",
            r#"
            - DOCUMENT@0..85
                - INPUT_OBJECT_TYPE_DEFINITION@0..85
                    - input_KW@0..5 "input"
                    - WHITESPACE@5..6 " "
                    - NAME@6..25
                        - IDENT@6..24 "ExampleInputObject"
                        - WHITESPACE@24..25 " "
                    - INPUT_FIELDS_DEFINITION@25..85
                        - L_CURLY@25..26 "{"
                        - WHITESPACE@26..41 "\n              "
                        - INPUT_VALUE_DEFINITION@41..65
                            - NAME@41..42
                                - IDENT@41..42 "a"
                            - COLON@42..43 ":"
                            - WHITESPACE@43..44 " "
                            - TYPE@44..65
                                - WHITESPACE@44..59 "\n              "
                                - NAMED_TYPE@59..65
                                    - NAME@59..65
                                        - IDENT@59..65 "String"
                        - INPUT_VALUE_DEFINITION@65..84
                            - NAME@65..66
                                - IDENT@65..66 "b"
                            - COLON@66..67 ":"
                            - WHITESPACE@67..68 " "
                            - TYPE@68..84
                                - WHITESPACE@68..81 "\n            "
                                - NON_NULL_TYPE@81..84
                                    - TYPE@81..84
                                        - NAMED_TYPE@81..84
                                            - NAME@81..84
                                                - IDENT@81..84 "Int"
                        - R_CURLY@84..85 "}"
            "#,
        )
    }

    #[test]
    fn it_creates_an_error_when_name_is_missing_in_definition() {
        utils::check_ast(
            "input {
              a: String
              b: Int!
            }",
            r#"
            - DOCUMENT@0..66
                - INPUT_OBJECT_TYPE_DEFINITION@0..66
                    - input_KW@0..5 "input"
                    - WHITESPACE@5..6 " "
                    - INPUT_FIELDS_DEFINITION@6..66
                        - L_CURLY@6..7 "{"
                        - WHITESPACE@7..22 "\n              "
                        - INPUT_VALUE_DEFINITION@22..46
                            - NAME@22..23
                                - IDENT@22..23 "a"
                            - COLON@23..24 ":"
                            - WHITESPACE@24..25 " "
                            - TYPE@25..46
                                - WHITESPACE@25..40 "\n              "
                                - NAMED_TYPE@40..46
                                    - NAME@40..46
                                        - IDENT@40..46 "String"
                        - INPUT_VALUE_DEFINITION@46..65
                            - NAME@46..47
                                - IDENT@46..47 "b"
                            - COLON@47..48 ":"
                            - WHITESPACE@48..49 " "
                            - TYPE@49..65
                                - WHITESPACE@49..62 "\n            "
                                - NON_NULL_TYPE@62..65
                                    - TYPE@62..65
                                        - NAMED_TYPE@62..65
                                            - NAME@62..65
                                                - IDENT@62..65 "Int"
                        - R_CURLY@65..66 "}"
            - ERROR@0:1 "expected a Name"
            "#,
        )
    }

    #[test]
    fn it_creates_an_error_when_enum_values_are_missing_in_definition() {
        utils::check_ast(
            "input ExampleInputObject {}",
            r#"
            - DOCUMENT@0..27
                - INPUT_OBJECT_TYPE_DEFINITION@0..27
                    - input_KW@0..5 "input"
                    - WHITESPACE@5..6 " "
                    - NAME@6..25
                        - IDENT@6..24 "ExampleInputObject"
                        - WHITESPACE@24..25 " "
                    - INPUT_FIELDS_DEFINITION@25..27
                        - L_CURLY@25..26 "{"
                        - R_CURLY@26..27 "}"
            - ERROR@0:1 "expected an Input Value Definition"
            "#,
        )
    }

    #[test]
    fn it_parses_extension() {
        utils::check_ast(
            "extend input ExampleInputObject @skip {
              a: String
            }",
            r#"
            - DOCUMENT@0..77
                - INPUT_OBJECT_TYPE_EXTENSION@0..77
                    - extend_KW@0..6 "extend"
                    - WHITESPACE@6..7 " "
                    - input_KW@7..12 "input"
                    - WHITESPACE@12..13 " "
                    - NAME@13..32
                        - IDENT@13..31 "ExampleInputObject"
                        - WHITESPACE@31..32 " "
                    - DIRECTIVES@32..38
                        - DIRECTIVE@32..38
                            - AT@32..33 "@"
                            - NAME@33..38
                                - IDENT@33..37 "skip"
                                - WHITESPACE@37..38 " "
                    - INPUT_FIELDS_DEFINITION@38..77
                        - L_CURLY@38..39 "{"
                        - WHITESPACE@39..54 "\n              "
                        - INPUT_VALUE_DEFINITION@54..76
                            - NAME@54..55
                                - IDENT@54..55 "a"
                            - COLON@55..56 ":"
                            - WHITESPACE@56..57 " "
                            - TYPE@57..76
                                - WHITESPACE@57..70 "\n            "
                                - NAMED_TYPE@70..76
                                    - NAME@70..76
                                        - IDENT@70..76 "String"
                        - R_CURLY@76..77 "}"
            "#,
        )
    }

    #[test]
    fn it_creates_an_error_when_name_is_missing_in_extension() {
        utils::check_ast(
            "extend input {
              a: String
            }",
            r#"
            - DOCUMENT@0..52
                - INPUT_OBJECT_TYPE_EXTENSION@0..52
                    - extend_KW@0..6 "extend"
                    - WHITESPACE@6..7 " "
                    - input_KW@7..12 "input"
                    - WHITESPACE@12..13 " "
                    - INPUT_FIELDS_DEFINITION@13..52
                        - L_CURLY@13..14 "{"
                        - WHITESPACE@14..29 "\n              "
                        - INPUT_VALUE_DEFINITION@29..51
                            - NAME@29..30
                                - IDENT@29..30 "a"
                            - COLON@30..31 ":"
                            - WHITESPACE@31..32 " "
                            - TYPE@32..51
                                - WHITESPACE@32..45 "\n            "
                                - NAMED_TYPE@45..51
                                    - NAME@45..51
                                        - IDENT@45..51 "String"
                        - R_CURLY@51..52 "}"
            - ERROR@0:1 "expected a Name"
            "#,
        )
    }

    #[test]
    fn it_creates_an_error_when_syntax_is_missing_in_extension() {
        utils::check_ast(
            "extend input ExampleInputObject",
            r#"
            - DOCUMENT@0..31
                - INPUT_OBJECT_TYPE_EXTENSION@0..31
                    - extend_KW@0..6 "extend"
                    - WHITESPACE@6..7 " "
                    - input_KW@7..12 "input"
                    - WHITESPACE@12..13 " "
                    - NAME@13..31
                        - IDENT@13..31 "ExampleInputObject"
            - ERROR@0:3 "expected Directives or an Input Fields Definition"
            "#,
        )
    }
}
