use crate::parser::grammar::{directive, name, ty, value};
use crate::{Parser, SyntaxKind, TokenKind, S, T};

/// See: https://spec.graphql.org/draft/#VariableDefinitions
///
/// *VariableDefinitions*:
///     **(** VariableDefinition<sub>list</sub> **)**
pub(crate) fn variable_definitions(p: &mut Parser) {
    let _g = p.start_node(SyntaxKind::VARIABLE_DEFINITIONS);
    p.bump(S!['(']);

    // TODO @lrlna error: expected a variable definition to follow an opening brace
    if let Some(T![$]) = p.peek() {
        variable_definition(p, false);
    }
    p.expect(T![')'], S![')']);
}

/// See: https://spec.graphql.org/draft/#VariableDefinition
///
/// *VariableDefinition*:
///     Variable **:** Type DefaultValue<sub>opt</sub> Directives<sub>\[Const\] opt</sub>
pub(crate) fn variable_definition(p: &mut Parser, is_variable: bool) {
    if let Some(T![$]) = p.peek() {
        let guard = p.start_node(SyntaxKind::VARIABLE_DEFINITION);
        variable(p);

        if let Some(T![:]) = p.peek() {
            p.bump(S![:]);
            if let Some(TokenKind::Name) = p.peek() {
                ty::ty(p);
                if let Some(T![=]) = p.peek() {
                    value::default_value(p);
                }
                if let Some(T![@]) = p.peek() {
                    directive::directives(p)
                }
                if p.peek().is_some() {
                    guard.finish_node();
                    return variable_definition(p, true);
                }
            }
            p.err("expected a Type");
        } else {
            p.err("expected a Name");
        }
    }

    if !is_variable {
        p.err("expected a Variable Definition");
    }
}

/// See: https://spec.graphql.org/draft/#Variable
///
/// *Variable*:
///     **$** Name
pub(crate) fn variable(p: &mut Parser) {
    let _g = p.start_node(SyntaxKind::VARIABLE);
    p.bump(S![$]);
    name::name(p);
}

#[cfg(test)]
mod test {
    use crate::parser::utils;

    #[test]
    fn it_parses_variables_with_default() {
        utils::check_ast(
            "query getOutput($input: Int = 5 $config: String = \"Config\"){
                animal
            }",
            r#"
            - DOCUMENT@0..97
                - OPERATION_DEFINITION@0..97
                    - OPERATION_TYPE@0..6
                        - query_KW@0..5 "query"
                        - WHITESPACE@5..6 " "
                    - NAME@6..15
                        - IDENT@6..15 "getOutput"
                    - VARIABLE_DEFINITIONS@15..59
                        - L_PAREN@15..16 "("
                        - VARIABLE_DEFINITION@16..32
                            - VARIABLE@16..22
                                - DOLLAR@16..17 "$"
                                - NAME@17..22
                                    - IDENT@17..22 "input"
                            - COLON@22..23 ":"
                            - WHITESPACE@23..24 " "
                            - TYPE@24..28
                                - WHITESPACE@24..25 " "
                                - NAMED_TYPE@25..28
                                    - NAME@25..28
                                        - IDENT@25..28 "Int"
                            - DEFAULT_VALUE@28..32
                                - EQ@28..29 "="
                                - WHITESPACE@29..30 " "
                                - VALUE@30..32
                                    - INT_VALUE@30..31 "5"
                                    - WHITESPACE@31..32 " "
                        - VARIABLE_DEFINITION@32..58
                            - VARIABLE@32..39
                                - DOLLAR@32..33 "$"
                                - NAME@33..39
                                    - IDENT@33..39 "config"
                            - COLON@39..40 ":"
                            - WHITESPACE@40..41 " "
                            - TYPE@41..48
                                - WHITESPACE@41..42 " "
                                - NAMED_TYPE@42..48
                                    - NAME@42..48
                                        - IDENT@42..48 "String"
                            - DEFAULT_VALUE@48..58
                                - EQ@48..49 "="
                                - WHITESPACE@49..50 " "
                                - VALUE@50..58
                                    - STRING_VALUE@50..58 "\"Config\""
                        - R_PAREN@58..59 ")"
                    - SELECTION_SET@59..97
                        - L_CURLY@59..60 "{"
                        - WHITESPACE@60..77 "\n                "
                        - SELECTION@77..96
                            - FIELD@77..96
                                - NAME@77..96
                                    - IDENT@77..83 "animal"
                                    - WHITESPACE@83..96 "\n            "
                        - R_CURLY@96..97 "}"
            "#,
        )
    }

    #[test]
    fn it_parses_operation_definition_with_arguments_and_directives() {
        utils::check_ast(
            "query myQuery($var: input $varOther: otherInput) @deprecated @unused {
                animal
                treat
            }",
            r#"
            - DOCUMENT@0..129
                - OPERATION_DEFINITION@0..129
                    - OPERATION_TYPE@0..6
                        - query_KW@0..5 "query"
                        - WHITESPACE@5..6 " "
                    - NAME@6..13
                        - IDENT@6..13 "myQuery"
                    - VARIABLE_DEFINITIONS@13..49
                        - L_PAREN@13..14 "("
                        - VARIABLE_DEFINITION@14..26
                            - VARIABLE@14..18
                                - DOLLAR@14..15 "$"
                                - NAME@15..18
                                    - IDENT@15..18 "var"
                            - COLON@18..19 ":"
                            - WHITESPACE@19..20 " "
                            - TYPE@20..26
                                - WHITESPACE@20..21 " "
                                - NAMED_TYPE@21..26
                                    - NAME@21..26
                                        - IDENT@21..26 "input"
                        - VARIABLE_DEFINITION@26..47
                            - VARIABLE@26..35
                                - DOLLAR@26..27 "$"
                                - NAME@27..35
                                    - IDENT@27..35 "varOther"
                            - COLON@35..36 ":"
                            - WHITESPACE@36..37 " "
                            - TYPE@37..47
                                - NAMED_TYPE@37..47
                                    - NAME@37..47
                                        - IDENT@37..47 "otherInput"
                        - R_PAREN@47..48 ")"
                        - WHITESPACE@48..49 " "
                    - DIRECTIVES@49..69
                        - DIRECTIVE@49..61
                            - AT@49..50 "@"
                            - NAME@50..61
                                - IDENT@50..60 "deprecated"
                                - WHITESPACE@60..61 " "
                        - DIRECTIVE@61..69
                            - AT@61..62 "@"
                            - NAME@62..69
                                - IDENT@62..68 "unused"
                                - WHITESPACE@68..69 " "
                    - SELECTION_SET@69..129
                        - L_CURLY@69..70 "{"
                        - WHITESPACE@70..87 "\n                "
                        - SELECTION@87..128
                            - FIELD@87..110
                                - NAME@87..110
                                    - IDENT@87..93 "animal"
                                    - WHITESPACE@93..110 "\n                "
                            - FIELD@110..128
                                - NAME@110..128
                                    - IDENT@110..115 "treat"
                                    - WHITESPACE@115..128 "\n            "
                        - R_CURLY@128..129 "}"
            "#,
        )
    }

    #[test]
    fn it_parses_operation_definition_with_arguments() {
        utils::check_ast(
            "query myQuery($var: input, $varOther: otherInput) {
                animal,
                treat
            }",
            r#"
            - DOCUMENT@0..111
                - OPERATION_DEFINITION@0..111
                    - OPERATION_TYPE@0..6
                        - query_KW@0..5 "query"
                        - WHITESPACE@5..6 " "
                    - NAME@6..13
                        - IDENT@6..13 "myQuery"
                    - VARIABLE_DEFINITIONS@13..50
                        - L_PAREN@13..14 "("
                        - VARIABLE_DEFINITION@14..27
                            - VARIABLE@14..18
                                - DOLLAR@14..15 "$"
                                - NAME@15..18
                                    - IDENT@15..18 "var"
                            - COLON@18..19 ":"
                            - WHITESPACE@19..20 " "
                            - TYPE@20..27
                                - COMMA@20..21 ","
                                - WHITESPACE@21..22 " "
                                - NAMED_TYPE@22..27
                                    - NAME@22..27
                                        - IDENT@22..27 "input"
                        - VARIABLE_DEFINITION@27..48
                            - VARIABLE@27..36
                                - DOLLAR@27..28 "$"
                                - NAME@28..36
                                    - IDENT@28..36 "varOther"
                            - COLON@36..37 ":"
                            - WHITESPACE@37..38 " "
                            - TYPE@38..48
                                - NAMED_TYPE@38..48
                                    - NAME@38..48
                                        - IDENT@38..48 "otherInput"
                        - R_PAREN@48..49 ")"
                        - WHITESPACE@49..50 " "
                    - SELECTION_SET@50..111
                        - L_CURLY@50..51 "{"
                        - WHITESPACE@51..68 "\n                "
                        - SELECTION@68..110
                            - FIELD@68..92
                                - NAME@68..92
                                    - IDENT@68..74 "animal"
                                    - COMMA@74..75 ","
                                    - WHITESPACE@75..92 "\n                "
                            - FIELD@92..110
                                - NAME@92..110
                                    - IDENT@92..97 "treat"
                                    - WHITESPACE@97..110 "\n            "
                        - R_CURLY@110..111 "}"
            "#,
        )
    }
}
