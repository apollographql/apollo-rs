use crate::{
    parser::grammar::{input, name, value},
    Parser, SyntaxKind, TokenKind, S, T,
};

/// See: https://spec.graphql.org/October2021/#Argument
///
/// *Argument*:
///    Name **:** Value
pub(crate) fn argument(p: &mut Parser, mut is_argument: bool) {
    if let Some(TokenKind::Name) = p.peek() {
        let guard = p.start_node(SyntaxKind::ARGUMENT);
        name::name(p);
        if let Some(T![:]) = p.peek() {
            p.bump(S![:]);
            value::value(p, false);
            is_argument = true;
            if p.peek().is_some() {
                guard.finish_node();
                return argument(p, is_argument);
            }
        }
    }
    if !is_argument {
        p.err("expected an Argument");
    }
}

/// See: https://spec.graphql.org/October2021/#Arguments
///
/// *Arguments*:
///    **(** Argument* **)**
pub(crate) fn arguments(p: &mut Parser) {
    let _g = p.start_node(SyntaxKind::ARGUMENTS);
    p.bump(S!['(']);
    argument(p, false);
    p.expect(T![')'], S![')']);
}

/// See: https://spec.graphql.org/October2021/#ArgumentsDefinition
///
/// *ArgumentsDefinition*:
///     **(** InputValueDefinition* **)**
pub(crate) fn arguments_definition(p: &mut Parser) {
    let _g = p.start_node(SyntaxKind::ARGUMENTS_DEFINITION);
    p.bump(S!['(']);
    input::input_value_definition(p, false);
    p.expect(T![')'], S![')']);
}

#[cfg(test)]
mod tests {
    use crate::ast::{self, AstNode};

    use super::*;

    #[test]
    fn it_can_access_arguments_in_fields() {
        let schema = r#"
type Query {
  bestSellers(category: ProductCategory = ALL): [Product] @join__field(graph: PRODUCTS)
  categories: [Department] @join__field(graph: PRODUCTS)
  product(id: ID!): Product @join__field(graph: PRODUCTS)
}
        "#;
        let parser = Parser::new(schema);
        let ast = parser.parse();

        assert!(ast.errors.is_empty());

        let document = ast.document();
        for definition in document.definitions() {
            if let ast::Definition::ObjectTypeDefinition(obj_def) = definition {
                for field in obj_def.fields_definition().unwrap().field_definitions() {
                    if field.name().unwrap().text() == "bestSellers" {
                        let argument = field
                            .arguments_definition()
                            .unwrap()
                            .input_value_definitions()
                            .into_iter()
                            .next()
                            .unwrap();
                        assert_eq!(argument.name().unwrap().text(), "category");
                        assert_eq!(argument.ty().unwrap().source_string(), "ProductCategory");
                        assert_eq!(
                            argument
                                .default_value()
                                .unwrap()
                                .value()
                                .unwrap()
                                .source_string(),
                            "ALL"
                        );
                    }
                }
            }
        }
    }
}
