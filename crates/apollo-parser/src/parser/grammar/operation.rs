use crate::parser::grammar::{directive, name, selection, ty, variable};
use crate::{Parser, SyntaxKind, TokenKind, S, T};

/// OperationTypeDefinition is used in a SchemaDefinition. Not to be confused
/// with OperationDefinition.
///
/// See: https://spec.graphql.org/June2018/#RootOperationTypeDefinition
///
/// ```txt
/// OperationTypeDefinition
///    OperationType : NamedType
/// ```
pub(crate) fn operation_type_definition(p: &mut Parser, is_operation_type: bool) {
    if let Some(TokenKind::Name) = p.peek() {
        let guard = p.start_node(SyntaxKind::OPERATION_TYPE_DEFINITION);
        operation_type(p);
        if let Some(T![:]) = p.peek() {
            p.bump(S![:]);
            ty::named_type(p);
            if p.peek().is_some() {
                guard.finish_node();
                return operation_type_definition(p, true);
            }
        } else {
            p.err("expected a Name Type");
        }
    }

    if !is_operation_type {
        p.err("expected an Operation Type");
    }
}

/// See: https://spec.graphql.org/June2018/#OperationDefinition
///
/// ```txt
/// OperationDefinition
///    OperationType Name VariableDefinitions Directives SelectionSet
///    Selection Set (TODO)
/// ```

pub(crate) fn operation_definition(p: &mut Parser) {
    let _g = p.start_node(SyntaxKind::OPERATION_DEFINITION);
    match p.peek() {
        Some(TokenKind::Name) => operation_type(p),
        Some(T!['{']) => selection::selection_set(p),
        _ => p.err("expected an Operation Type or a Selection Set"),
    }
    if let Some(TokenKind::Name) = p.peek() {
        name::name(p);
    }

    if let Some(T!['(']) = p.peek() {
        let _g = p.start_node(SyntaxKind::VARIABLE_DEFINITIONS);
        p.bump(S!['(']);
        if let Some(T![$]) = p.peek() {
            variable::variable_definition(p, false);
        }
        p.expect(T![')'], S![')']);
        // TODO @lrlna error: expected a variable definition to follow an opening brace
    }
    if let Some(T![@]) = p.peek() {
        directive::directives(p);
    }
    if let Some(T!['{']) = p.peek() {
        selection::selection_set(p)
    }
}

/// See: https://spec.graphql.org/June2018/#OperationType
///
/// ```txt
/// OperationType : one of
///    query    mutation    subscription
/// ```
pub(crate) fn operation_type(p: &mut Parser) {
    if let Some(node) = p.peek_data() {
        let _g = p.start_node(SyntaxKind::OPERATION_TYPE);
        match node.as_str() {
            "query" => p.bump(SyntaxKind::query_KW),
            "subscription" => p.bump(SyntaxKind::subscription_KW),
            "mutation" => p.bump(SyntaxKind::mutation_KW),
            _ => p.err("expected either a 'mutation', a 'query', or a 'subscription'"),
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
            - DOCUMENT@0..215
                - OPERATION_DEFINITION@0..215
                    - OPERATION_TYPE@0..6
                        - query_KW@0..5 "query"
                        - WHITESPACE@5..6 " "
                    - NAME@6..14
                        - IDENT@6..13 "myQuery"
                        - WHITESPACE@13..14 " "
                    - SELECTION_SET@14..215
                        - L_CURLY@14..15 "{"
                        - WHITESPACE@15..32 "\n                "
                        - SELECTION@32..214
                            - FIELD@32..60
                                - ALIAS@32..40
                                    - NAME@32..38
                                        - IDENT@32..38 "animal"
                                    - COLON@38..39 ":"
                                    - WHITESPACE@39..40 " "
                                - NAME@40..60
                                    - IDENT@40..43 "cat"
                                    - WHITESPACE@43..60 "\n                "
                            - FIELD@60..197
                                - NAME@60..64
                                    - IDENT@60..63 "dog"
                                    - WHITESPACE@63..64 " "
                                - SELECTION_SET@64..180
                                    - L_CURLY@64..65 "{"
                                    - WHITESPACE@65..86 "\n                    "
                                    - SELECTION@86..179
                                        - FIELD@86..179
                                            - NAME@86..92
                                                - IDENT@86..91 "panda"
                                                - WHITESPACE@91..92 " "
                                            - SELECTION_SET@92..162
                                                - L_CURLY@92..93 "{"
                                                - WHITESPACE@93..118 "\n                        "
                                                - SELECTION@118..161
                                                    - FIELD@118..161
                                                        - NAME@118..129
                                                            - IDENT@118..128 "anotherCat"
                                                            - WHITESPACE@128..129 " "
                                                        - DIRECTIVES@129..161
                                                            - DIRECTIVE@129..161
                                                                - AT@129..130 "@"
                                                                - NAME@130..161
                                                                    - IDENT@130..140 "deprecated"
                                                                    - WHITESPACE@140..161 "\n                    "
                                                - R_CURLY@161..162 "}"
                                            - WHITESPACE@162..179 "\n                "
                                    - R_CURLY@179..180 "}"
                                - WHITESPACE@180..197 "\n                "
                            - FIELD@197..214
                                - NAME@197..214
                                    - IDENT@197..201 "lion"
                                    - WHITESPACE@201..214 "\n            "
                        - R_CURLY@214..215 "}"
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
            - DOCUMENT@0..108
                - OPERATION_DEFINITION@0..108
                    - OPERATION_TYPE@0..6
                        - query_KW@0..5 "query"
                        - WHITESPACE@5..6 " "
                    - NAME@6..13
                        - IDENT@6..13 "myQuery"
                    - VARIABLE_DEFINITIONS@13..48
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
                    - SELECTION_SET@48..108
                        - L_CURLY@48..49 "{"
                        - WHITESPACE@49..66 "\n                "
                        - SELECTION@66..107
                            - FIELD@66..89
                                - NAME@66..89
                                    - IDENT@66..72 "animal"
                                    - WHITESPACE@72..89 "\n                "
                            - FIELD@89..107
                                - NAME@89..107
                                    - IDENT@89..94 "treat"
                                    - WHITESPACE@94..107 "\n            "
                        - R_CURLY@107..108 "}"
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
                    - VARIABLE_DEFINITIONS@13..48
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
}
