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

/// See: https://spec.graphql.org/October2021/#OperationDefinition
///
/// *OperationDefinition*:
///    OperationType Name? VariableDefinitions? Directives? SelectionSet
///    SelectionSet
pub(crate) fn operation_definition(p: &mut Parser) {
    let _g = p.start_node(SyntaxKind::OPERATION_DEFINITION);

    let description_token = p.peek_token()
        .filter(|token| token.kind() == TokenKind::StringValue)
        .cloned();
    if description_token.is_some() {
        description::description(p);
    }

    match p.peek() {
        Some(TokenKind::Name) => {
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
            if let Some(description_token) = description_token {
                p.err_at_token(&description_token, "shorthand operation must not have a description");
            }
            selection::selection_set(p)
        }
        _ => p.err_and_pop("expected an Operation Type or a Selection Set"),
    }
}

/// See: https://spec.graphql.org/October2021/#OperationType
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

    // NOTE @lrlna: related PR to the spec to avoid this issue:
    // https://github.com/graphql/graphql-spec/pull/892
    #[test]
    fn it_continues_parsing_when_operation_definition_starts_with_description() {
        let input = "\"description\"{}";
        let parser = Parser::new(input);
        let cst = parser.parse();

        assert_eq!(cst.errors().len(), 2);
        assert_eq!(cst.document().definitions().count(), 1);
    }
}
