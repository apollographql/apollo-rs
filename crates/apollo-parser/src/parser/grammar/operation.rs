use crate::parser::grammar::description;
use crate::parser::grammar::directive;
use crate::parser::grammar::name;
use crate::parser::grammar::selection;
use crate::parser::grammar::value::Constness;
use crate::parser::grammar::variable;
use crate::Parser;
use crate::SyntaxKind;
use crate::TokenKind;
use crate::T;

/// See: https://spec.graphql.org/September2025/#sec-Language.Operations
///
/// *OperationDefinition*:
///    Description? OperationType Name? VariableDefinitions? Directives? SelectionSet
///    SelectionSet
pub(crate) fn operation_definition(p: &mut Parser) {
    match p.peek() {
        Some(TokenKind::StringValue) => {
            // Description found - must be full operation definition, not shorthand
            let _g = p.start_node(SyntaxKind::OPERATION_DEFINITION);

            description::description(p);

            // After description, we must have an operation type
            if let Some(TokenKind::Name) = p.peek() {
                operation_type(p);
            } else {
                return p.err_and_pop("expected an Operation Type after description");
            }

            if let Some(TokenKind::Name) = p.peek() {
                name::name(p);
            }

            if let Some(T!['(']) = p.peek() {
                variable::variable_definitions(p)
            }

            if let Some(T![@]) = p.peek() {
                directive::directives(p, Constness::NotConst);
            }

            match p.peek() {
                Some(T!['{']) => selection::selection_set(p),
                _ => p.err_and_pop("expected a Selection Set"),
            }
        }
        Some(TokenKind::Name) => {
            let _g = p.start_node(SyntaxKind::OPERATION_DEFINITION);

            operation_type(p);

            if let Some(TokenKind::Name) = p.peek() {
                name::name(p);
            }

            if let Some(T!['(']) = p.peek() {
                variable::variable_definitions(p)
            }

            if let Some(T![@]) = p.peek() {
                directive::directives(p, Constness::NotConst);
            }

            match p.peek() {
                Some(T!['{']) => selection::selection_set(p),
                _ => p.err_and_pop("expected a Selection Set"),
            }
        }
        Some(T!['{']) => {
            let _g = p.start_node(SyntaxKind::OPERATION_DEFINITION);

            selection::selection_set(p)
        }
        _ => p.err_and_pop("expected an Operation Type or a Selection Set"),
    }
}

/// See: https://spec.graphql.org/September2025/#sec-Language.Operations
///
/// *OperationType*: one of
///    **query**    **mutation**    **subscription**
pub(crate) fn operation_type(p: &mut Parser) {
    if let Some(node) = p.peek_data() {
        let _g = p.start_node(SyntaxKind::OPERATION_TYPE);
        match node {
            "query" => p.bump(SyntaxKind::query_KW),
            "subscription" => p.bump(SyntaxKind::subscription_KW),
            "mutation" => p.bump(SyntaxKind::mutation_KW),
            _ => p.err_and_pop("expected either a 'mutation', a 'query', or a 'subscription'"),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::Parser;

    // NOTE @lrlna: Descriptions on operations are now supported in September 2025 spec
    // https://github.com/graphql/graphql-spec/pull/1170
    #[test]
    fn it_parses_operation_definition_with_description() {
        let input = "\"A test query\" query Test { field }";
        let parser = Parser::new(input);
        let cst = parser.parse();

        assert_eq!(cst.errors().len(), 0);
        assert_eq!(cst.document().definitions().count(), 1);
    }

    #[test]
    fn it_errors_on_shorthand_query_with_description() {
        let input = "\"description\"{}";
        let parser = Parser::new(input);
        let cst = parser.parse();

        // Should error because descriptions are not allowed on shorthand queries
        assert!(cst.errors().len() > 0);
    }
}
