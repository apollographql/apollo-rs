use crate::parser::grammar::{description, directive, operation};
use crate::{Parser, SyntaxKind, TokenKind, S, T};

/// See: https://spec.graphql.org/June2018/#SchemaDefinition
///
/// ```txt
/// SchemaDefinition
///    Description[opt] schema Directives[Const][opt] { RootOperationTypeDefinition[list] }
/// ```
pub(crate) fn schema_definition(p: &mut Parser) {
    let _g = p.start_node(SyntaxKind::SCHEMA_DEFINITION);

    if let Some(TokenKind::StringValue) = p.peek() {
        description::description(p);
    }

    if let Some("schema") = p.peek_data().as_deref() {
        p.bump(SyntaxKind::schema_KW);
    }

    if let Some(T![@]) = p.peek() {
        directive::directives(p);
    }

    if let Some(T!['{']) = p.peek() {
        p.bump(S!['{']);
        operation::operation_type_definition(p, false);
        p.expect(T!['}'], S!['}']);
    } else {
        p.err("expected Root Operation Type Definition");
    }
}

/// See: https://spec.graphql.org/June2018/#SchemaExtension
///
/// ```txt
/// SchemaExtension
///     extend schema Directives[Const][opt] { OperationTypeDefinition[list] }
///     extend schema Directives[Const]
/// ```
pub(crate) fn schema_extension(p: &mut Parser) {
    let _g = p.start_node(SyntaxKind::SCHEMA_EXTENSION);
    p.bump(SyntaxKind::extend_KW);
    p.bump(SyntaxKind::schema_KW);

    if let Some(T![@]) = p.peek() {
        directive::directives(p);
    }

    if let Some(T!['{']) = p.peek() {
        p.bump(S!['{']);
        operation::operation_type_definition(p, false);
        p.expect(T!['}'], S!['}']);
    }
}

// TODO @lrlna: inlined collapsed AST should live in a 'fixtures' dir for ease of testing
#[cfg(test)]
mod test {
    use crate::parser::utils;

    #[test]
    fn it_parses_schema_definition() {
        utils::check_ast(
            "schema {
                query: MyQueryRootType
                mutation: MyMutationRootType,
                subscription: MySubscriptionRootType
            }",
            r#"
            - DOCUMENT@0..160
                - SCHEMA_DEFINITION@0..160
                    - schema_KW@0..6 "schema"
                    - WHITESPACE@6..7 " "
                    - L_CURLY@7..8 "{"
                    - WHITESPACE@8..25 "\n                "
                    - OPERATION_TYPE_DEFINITION@25..64
                        - OPERATION_TYPE@25..30
                            - query_KW@25..30 "query"
                        - COLON@30..31 ":"
                        - WHITESPACE@31..32 " "
                        - NAMED_TYPE@32..64
                            - NAME@32..64
                                - IDENT@32..47 "MyQueryRootType"
                                - WHITESPACE@47..64 "\n                "
                    - OPERATION_TYPE_DEFINITION@64..92
                        - OPERATION_TYPE@64..72
                            - mutation_KW@64..72 "mutation"
                        - COLON@72..73 ":"
                        - WHITESPACE@73..74 " "
                        - NAMED_TYPE@74..92
                            - NAME@74..92
                                - IDENT@74..92 "MyMutationRootType"
                    - COMMA@92..93 ","
                    - WHITESPACE@93..110 "\n                "
                    - OPERATION_TYPE_DEFINITION@110..159
                        - OPERATION_TYPE@110..122
                            - subscription_KW@110..122 "subscription"
                        - COLON@122..123 ":"
                        - WHITESPACE@123..124 " "
                        - NAMED_TYPE@124..159
                            - NAME@124..159
                                - IDENT@124..146 "MySubscriptionRootType"
                                - WHITESPACE@146..159 "\n            "
                    - R_CURLY@159..160 "}"
            "#,
        );
    }

    #[test]
    fn it_parses_schema_extension() {
        utils::check_ast(
            "extend schema @skip @example {
                query: MyExtendedQueryType
            }",
            r#"
            - DOCUMENT@0..87
                - SCHEMA_EXTENSION@0..87
                    - extend_KW@0..6 "extend"
                    - WHITESPACE@6..7 " "
                    - schema_KW@7..13 "schema"
                    - WHITESPACE@13..14 " "
                    - DIRECTIVES@14..29
                        - DIRECTIVE@14..20
                            - AT@14..15 "@"
                            - NAME@15..20
                                - IDENT@15..19 "skip"
                                - WHITESPACE@19..20 " "
                        - DIRECTIVE@20..29
                            - AT@20..21 "@"
                            - NAME@21..29
                                - IDENT@21..28 "example"
                                - WHITESPACE@28..29 " "
                    - L_CURLY@29..30 "{"
                    - WHITESPACE@30..47 "\n                "
                    - OPERATION_TYPE_DEFINITION@47..86
                        - OPERATION_TYPE@47..52
                            - query_KW@47..52 "query"
                        - COLON@52..53 ":"
                        - WHITESPACE@53..54 " "
                        - NAMED_TYPE@54..86
                            - NAME@54..86
                                - IDENT@54..73 "MyExtendedQueryType"
                                - WHITESPACE@73..86 "\n            "
                    - R_CURLY@86..87 "}"
            "#,
        );
    }
}
