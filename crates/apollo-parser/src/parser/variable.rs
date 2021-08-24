use crate::parser::{name, ty, value};
use crate::{create_err, Parser, SyntaxKind, TokenKind};

/// See: https://spec.graphql.org/June2018/#VariableDefinition
///
/// ```txt
/// VariableDefinition
///     Variable : Type DefaultValue(opt)
/// ```
pub(crate) fn variable_definition(parser: &mut Parser, is_variable: bool) {
    if let Some(TokenKind::Dollar) = parser.peek() {
        let guard = parser.start_node(SyntaxKind::VARIABLE_DEFINITION);
        variable(parser);
        if let Some(TokenKind::Colon) = parser.peek() {
            parser.bump(SyntaxKind::COLON);
            if let Some(TokenKind::Node) = parser.peek() {
                ty::ty(parser);
                if let Some(TokenKind::Eq) = parser.peek() {
                    value::default_value(parser);
                }
                if parser.peek().is_some() {
                    guard.finish_node();
                    return variable_definition(parser, true);
                }
            }
            parser.push_err(create_err!(
                parser.peek_data().unwrap(),
                "Expected Variable Definition to have a Type, got {}",
                parser.peek_data().unwrap()
            ));
        } else {
            parser.push_err(create_err!(
                parser.peek_data().unwrap(),
                "Expected Variable Definition to have a Name, got {}",
                parser.peek_data().unwrap()
            ));
        }
    }

    if let Some(TokenKind::Comma) = parser.peek() {
        parser.bump(SyntaxKind::COMMA);
        return variable_definition(parser, is_variable);
    }

    if !is_variable {
        parser.push_err(create_err!(
            parser.peek_data().unwrap(),
            "Expected to have an Variable Definition, got {}",
            parser.peek_data().unwrap()
        ));
    }
}

/// See: https://spec.graphql.org/June2018/#Variable
///
/// ```txt
/// Variable
///     $ Name
/// ```
pub(crate) fn variable(parser: &mut Parser) {
    let _guard = parser.start_node(SyntaxKind::VARIABLE);
    parser.bump(SyntaxKind::DOLLAR);
    name::name(parser);
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
            - DOCUMENT@0..50
                - OPERATION_DEFINITION@0..50
                    - OPERATION_TYPE@0..5
                        - query_KW@0..5 "query"
                    - NAME@5..14
                        - IDENT@5..14 "getOutput"
                    - VARIABLE_DEFINITIONS@14..42
                        - L_PAREN@14..15 "("
                        - VARIABLE_DEFINITION@15..24
                            - VARIABLE@15..21
                                - DOLLAR@15..16 "$"
                                - NAME@16..21
                                    - IDENT@16..21 "input"
                            - COLON@21..22 ":"
                            - TYPE@22..22
                                - NAMED_TYPE@22..22
                            - DEFAULT_VALUE@22..24
                                - EQ@22..23 "="
                                - VALUE@23..24
                                    - INT_VALUE@23..24 "5"
                        - VARIABLE_DEFINITION@24..41
                            - VARIABLE@24..31
                                - DOLLAR@24..25 "$"
                                - NAME@25..31
                                    - IDENT@25..31 "config"
                            - COLON@31..32 ":"
                            - TYPE@32..32
                                - NAMED_TYPE@32..32
                            - DEFAULT_VALUE@32..41
                                - EQ@32..33 "="
                                - VALUE@33..41
                                    - STRING_VALUE@33..41 "\"Config\""
                        - R_PAREN@41..42 ")"
                    - SELECTION_SET@42..50
                        - L_CURLY@42..43 "{"
                        - SELECTION@43..49
                            - FIELD@43..49
                                - NAME@43..49
                                    - IDENT@43..49 "animal"
                        - R_CURLY@49..50 "}"
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
            - DOCUMENT@0..60
                - OPERATION_DEFINITION@0..60
                    - OPERATION_TYPE@0..5
                        - query_KW@0..5 "query"
                    - NAME@5..12
                        - IDENT@5..12 "myQuery"
                    - VARIABLE_DEFINITIONS@12..29
                        - L_PAREN@12..13 "("
                        - VARIABLE_DEFINITION@13..18
                            - VARIABLE@13..17
                                - DOLLAR@13..14 "$"
                                - NAME@14..17
                                    - IDENT@14..17 "var"
                            - COLON@17..18 ":"
                            - TYPE@18..18
                                - NAMED_TYPE@18..18
                        - VARIABLE_DEFINITION@18..28
                            - VARIABLE@18..27
                                - DOLLAR@18..19 "$"
                                - NAME@19..27
                                    - IDENT@19..27 "varOther"
                            - COLON@27..28 ":"
                            - TYPE@28..28
                                - NAMED_TYPE@28..28
                        - R_PAREN@28..29 ")"
                    - DIRECTIVES@29..47
                        - DIRECTIVE@29..40
                            - AT@29..30 "@"
                            - NAME@30..40
                                - IDENT@30..40 "deprecated"
                        - DIRECTIVE@40..47
                            - AT@40..41 "@"
                            - NAME@41..47
                                - IDENT@41..47 "unused"
                    - SELECTION_SET@47..60
                        - L_CURLY@47..48 "{"
                        - SELECTION@48..59
                            - FIELD@48..54
                                - NAME@48..54
                                    - IDENT@48..54 "animal"
                            - FIELD@54..59
                                - NAME@54..59
                                    - IDENT@54..59 "treat"
                        - R_CURLY@59..60 "}"
            "#,
        )
    }
}
