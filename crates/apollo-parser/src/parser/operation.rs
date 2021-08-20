use crate::parser::{directive, name, selection, ty, variable};
use crate::{create_err, Parser, SyntaxKind, TokenKind};

/// OperationTypeDefinition is used in a SchemaDefinition. Not to be confused
/// with OperationDefinition.
///
/// See: https://spec.graphql.org/June2018/#RootOperationTypeDefinition
///
/// ```txt
/// OperationTypeDefinition
///    OperationType : NamedType
/// ```
pub(crate) fn operation_type_definition(parser: &mut Parser, is_operation_type: bool) {
    if let Some(TokenKind::Comma) = parser.peek() {
        parser.bump(SyntaxKind::COMMA);
        return operation_type_definition(parser, is_operation_type);
    }

    if let Some(TokenKind::Node) = parser.peek() {
        let guard = parser.start_node(SyntaxKind::OPERATION_TYPE_DEFINITION);
        operation_type(parser);
        if let Some(TokenKind::Colon) = parser.peek() {
            parser.bump(SyntaxKind::COLON);
            ty::named_type(parser);
            if parser.peek().is_some() {
                guard.finish_node();
                return operation_type_definition(parser, true);
            }
        } else {
            parser.push_err(create_err!(
                parser.peek_data().unwrap(),
                "Expected Operation Type to have a Name Type, got {}",
                parser.peek_data().unwrap()
            ));
        }
    }

    if !is_operation_type {
        parser.push_err(create_err!(
            parser.peek_data().unwrap(),
            "Expected Schema Definition to have an Operation Type, got {}",
            parser.peek_data().unwrap()
        ));
    }
}

/// See: https://spec.graphql.org/June2018/#OperationDefinition
///
/// ```txt
/// OperationDefinition
///    OperationType Name VariableDefinitions Directives SelectionSet
///    Selection Set (TODO)
/// ```

pub(crate) fn operation_definition(parser: &mut Parser) {
    let _guard = parser.start_node(SyntaxKind::OPERATION_DEFINITION);
    operation_type(parser);
    if let Some(TokenKind::Node) = parser.peek() {
        name::name(parser);
    }

    if let Some(TokenKind::LParen) = parser.peek() {
        let guard = parser.start_node(SyntaxKind::VARIABLE_DEFINITIONS);
        parser.bump(SyntaxKind::L_PAREN);
        if let Some(TokenKind::Dollar) = parser.peek() {
            variable::variable_definition(parser, false);
        }
        if let Some(TokenKind::RParen) = parser.peek() {
            parser.bump(SyntaxKind::R_PAREN);
            guard.finish_node();
        }
        // TODO @lrlna error: expected a variable definition to follow an opening brace
    }
    if let Some(TokenKind::At) = parser.peek() {
        directive::directives(parser);
    }
    if let Some(TokenKind::LCurly) = parser.peek() {
        selection::selection_set(parser);
    }
}

/// See: https://spec.graphql.org/June2018/#OperationType
///
/// ```txt
/// OperationType : one of
///    query    mutation    subscription
/// ```
pub(crate) fn operation_type(parser: &mut Parser) {
    if let Some(node) = parser.peek_data() {
        let _guard = parser.start_node(SyntaxKind::OPERATION_TYPE);
        match node.as_str() {
            "query" => parser.bump(SyntaxKind::query_KW),
            "subscription" => parser.bump(SyntaxKind::subscription_KW),
            "mutation" => parser.bump(SyntaxKind::mutation_KW),
            _ => parser.push_err(create_err!(
                parser
                    .peek_data()
                    .unwrap_or_else(|| String::from("no further data")),
                "Operation Type must be either 'mutation', 'query' or 'subscription', got {}",
                parser
                    .peek_data()
                    .unwrap_or_else(|| String::from("no further data"))
            )),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::parser::utils;

    #[test]
    fn it_parses_operation_definition() {
        utils::check_ast(
            "query myQuery {
                animal: cat
                dog {
                    panda {
                        anotherCat @deprecated
                    }
                }
                lion
            }",
            r#"
            - DOCUMENT@0..61
                - OPERATION_DEFINITION@0..61
                    - OPERATION_TYPE@0..5
                        - query_KW@0..5 "query"
                    - NAME@5..12
                        - IDENT@5..12 "myQuery"
                    - SELECTION_SET@12..61
                        - L_CURLY@12..13 "{"
                        - SELECTION@13..60
                            - FIELD@13..23
                                - ALIAS@13..20
                                    - NAME@13..19
                                        - IDENT@13..19 "animal"
                                    - COLON@19..20 ":"
                                - NAME@20..23
                                    - IDENT@20..23 "cat"
                            - FIELD@23..56
                                - NAME@23..26
                                    - IDENT@23..26 "dog"
                                - SELECTION_SET@26..56
                                    - L_CURLY@26..27 "{"
                                    - SELECTION@27..55
                                        - FIELD@27..55
                                            - NAME@27..32
                                                - IDENT@27..32 "panda"
                                            - SELECTION_SET@32..55
                                                - L_CURLY@32..33 "{"
                                                - SELECTION@33..54
                                                    - FIELD@33..54
                                                        - NAME@33..43
                                                            - IDENT@33..43 "anotherCat"
                                                        - DIRECTIVES@43..54
                                                            - DIRECTIVE@43..54
                                                                - AT@43..44 "@"
                                                                - NAME@44..54
                                                                    - IDENT@44..54 "deprecated"
                                                - R_CURLY@54..55 "}"
                                    - R_CURLY@55..56 "}"
                            - FIELD@56..60
                                - NAME@56..60
                                    - IDENT@56..60 "lion"
                        - R_CURLY@60..61 "}"
            "#,
        )
    }

    #[test]
    fn it_parses_operation_definition_with_arguments() {
        utils::check_ast(
            "query myQuery($var: input $varOther: otherInput){
                animal
                treat
            }",
            r#"
            - DOCUMENT@0..42
                - OPERATION_DEFINITION@0..42
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
                    - SELECTION_SET@29..42
                        - L_CURLY@29..30 "{"
                        - SELECTION@30..41
                            - FIELD@30..36
                                - NAME@30..36
                                    - IDENT@30..36 "animal"
                            - FIELD@36..41
                                - NAME@36..41
                                    - IDENT@36..41 "treat"
                        - R_CURLY@41..42 "}"
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
