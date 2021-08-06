use crate::{directive, format_err, operation_type_definition, Parser, SyntaxKind, TokenKind};

/// See: https://spec.graphql.org/June2018/#SchemaDefinition
///
/// ```txt
/// SchemaDefinition
///    schema Directives { OperationTypeDefinition }
/// ```
pub(crate) fn schema_definition(parser: &mut Parser) -> Result<(), crate::Error> {
    let _guard = parser.start_node(SyntaxKind::SCHEMA_DEFINITION);

    parser.bump(SyntaxKind::schema_KW);

    if let Some(TokenKind::LParen) = parser.peek() {
        parser.bump(SyntaxKind::L_PAREN);
        directive(parser)?;
        if let Some(TokenKind::RParen) = parser.peek() {
            parser.bump(SyntaxKind::R_PAREN);
        } else {
            return format_err!(
                parser.peek_data().unwrap(),
                "Expected closing ')' parenthesis, got {}",
                parser.peek_data().unwrap()
            );
        }
    }

    if let Some(TokenKind::LCurly) = parser.peek() {
        parser.bump(SyntaxKind::L_CURLY);
        operation_type_definition(parser, false)?;
        if let Some(TokenKind::RCurly) = parser.peek() {
            parser.bump(SyntaxKind::R_CURLY);
        } else {
            return format_err!(
                parser.peek_data().unwrap(),
                "Expected Schema Definition to have a closing curly bracket, got {}",
                parser.peek_data().unwrap()
            );
        }
    } else {
        return format_err!(
            parser.peek_data().unwrap(),
            "Expected Schema Definition to define a root operation, got {}",
            parser.peek_data().unwrap()
        );
    }

    Ok(())
}

// TODO @lrlna: inlined collapsed AST should live in a 'fixtures' dir for ease of testing
#[cfg(test)]
mod test {
    use super::*;
    use indoc::indoc;
    use pretty_assertions::assert_eq;

    #[test]
    fn smoke_schema_definition() {
        let input = "schema { query: MyQueryRootType mutation: MyMutationRootType }";
        let parser = Parser::new(input);

        println!("{:?}", parser.parse());
    }

    // TODO @lrlna: these tests need to check for indentation as part of the
    // output, not just the nodes of the tree
    #[test]
    fn it_parses_schema_definition() {
        let input = "schema { query: MyQueryRootType, mutation: MyMutationRootType, subscription: MySubscriptionRootType }";
        let parser = Parser::new(input);
        let output = parser.parse();

        assert!(output.errors().is_empty());
        assert_eq!(
            format!("{:?}", output),
            indoc! { r#"
            - DOCUMENT@0..93
            - SCHEMA_DEFINITION@0..93
            - schema_KW@0..6 "schema"
            - L_CURLY@6..7 "{"
            - OPERATION_TYPE_DEFINITION@7..28
            - OPERATION_TYPE@7..12
            - query_KW@7..12 "query"
            - COLON@12..13 ":"
            - NAMED_TYPE@13..28
            - NAME@13..28
            - IDENT@13..28 "MyQueryRootType"
            - COMMA@28..29 ","
            - OPERATION_TYPE_DEFINITION@29..56
            - OPERATION_TYPE@29..37
            - mutation_KW@29..37 "mutation"
            - COLON@37..38 ":"
            - NAMED_TYPE@38..56
            - NAME@38..56
            - IDENT@38..56 "MyMutationRootType"
            - COMMA@56..57 ","
            - OPERATION_TYPE_DEFINITION@57..92
            - OPERATION_TYPE@57..69
            - subscription_KW@57..69 "subscription"
            - COLON@69..70 ":"
            - NAMED_TYPE@70..92
            - NAME@70..92
            - IDENT@70..92 "MySubscriptionRootType"
            - R_CURLY@92..93 "}"
            "# }
        );
    }
}
