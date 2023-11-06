use crate::{
    parser::grammar::{input, name, value},
    Parser, SyntaxKind, TokenKind, S, T,
};

/// See: https://spec.graphql.org/October2021/#Argument
///
/// *Argument*:
///    Name **:** Value
pub(crate) fn argument(p: &mut Parser) {
    let _guard = p.start_node(SyntaxKind::ARGUMENT);
    name::name(p);
    if let Some(T![:]) = p.peek() {
        p.bump(S![:]);
        value::value(p, false);
    }
}

/// See: https://spec.graphql.org/October2021/#Arguments
///
/// *Arguments*:
///    **(** Argument* **)**
pub(crate) fn arguments(p: &mut Parser) {
    let _g = p.start_node(SyntaxKind::ARGUMENTS);
    p.bump(S!['(']);
    if let Some(TokenKind::Name) = p.peek() {
        argument(p);
    } else {
        p.err("expected an Argument");
    }
    while let Some(TokenKind::Name) = p.peek() {
        argument(p);
    }
    p.expect(T![')'], S![')']);
}

/// See: https://spec.graphql.org/October2021/#ArgumentsDefinition
///
/// *ArgumentsDefinition*:
///     **(** InputValueDefinition* **)**
pub(crate) fn arguments_definition(p: &mut Parser) {
    let _g = p.start_node(SyntaxKind::ARGUMENTS_DEFINITION);
    p.bump(S!['(']);
    if let Some(TokenKind::Name | TokenKind::StringValue) = p.peek() {
        input::input_value_definition(p);
    } else {
        p.err("expected an Argument Definition");
    }
    while let Some(TokenKind::Name | TokenKind::StringValue) = p.peek() {
        input::input_value_definition(p);
    }
    p.expect(T![')'], S![')']);
}

#[cfg(test)]
mod tests {
    use crate::cst::{self, CstNode};

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
        let cst = parser.parse();

        assert!(cst.errors.is_empty());

        let document = cst.document();
        for definition in document.definitions() {
            if let cst::Definition::ObjectTypeDefinition(obj_def) = definition {
                for field in obj_def.fields_definition().unwrap().field_definitions() {
                    if field.name().unwrap().text() == "bestSellers" {
                        let argument = field
                            .arguments_definition()
                            .unwrap()
                            .input_value_definitions()
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
