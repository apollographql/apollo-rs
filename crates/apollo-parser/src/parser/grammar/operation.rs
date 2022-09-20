use crate::{
    parser::grammar::{directive, name, selection, ty, variable},
    Parser, SyntaxKind, TokenKind, S, T,
};

/// RootOperationTypeDefinition is used in a SchemaDefinition. Not to be confused
/// with OperationDefinition.
///
/// See: https://spec.graphql.org/October2021/#RootOperationTypeDefinition
///
/// *RootOperationTypeDefinition*:
///    OperationType **:** NamedType
pub(crate) fn root_operation_type_definition(p: &mut Parser, is_operation_type: bool) {
    if let Some(T!['{']) = p.peek() {
        p.bump(S!['{']);
    }

    if let Some(TokenKind::Name) = p.peek() {
        let guard = p.start_node(SyntaxKind::ROOT_OPERATION_TYPE_DEFINITION);
        operation_type(p);
        if let Some(T![:]) = p.peek() {
            p.bump(S![:]);
            ty::named_type(p);
            if p.peek().is_some() {
                guard.finish_node();
                return root_operation_type_definition(p, true);
            }
        } else {
            p.err("expected a Name Type");
        }
    }

    if !is_operation_type {
        p.err("expected an Operation Type");
    }
}

/// See: https://spec.graphql.org/October2021/#OperationDefinition
///
/// *OperationDefinition*:
///    OperationType Name? VariableDefinitions? Directives? SelectionSet
///    SelectionSet

pub(crate) fn operation_definition(p: &mut Parser) {
    match p.peek() {
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
                directive::directives(p);
            }

            match p.peek() {
                Some(T!['{']) => selection::top_selection_set(p),
                _ => p.err_and_pop("expected a Selection Set"),
            }
        }
        Some(T!['{']) => {
            let _g = p.start_node(SyntaxKind::OPERATION_DEFINITION);

            selection::top_selection_set(p)
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
    use crate::Parser;

    // NOTE @lrlna: related PR to the spec to avoid this issue:
    // https://github.com/graphql/graphql-spec/pull/892
    #[test]
    fn it_continues_parsing_when_operation_definition_starts_with_description() {
        let input = "\"description\"{}";
        let parser = Parser::new(input);
        let ast = parser.parse();

        assert_eq!(ast.errors().len(), 2);
        assert_eq!(ast.document().definitions().into_iter().count(), 1);
    }
}
