use crate::{Parser, SyntaxKind};

/// See: https://spec.graphql.org/October2021/#Description
///
/// *Description*:
///     StringValue
pub(crate) fn description(p: &mut Parser) {
    let _g = p.start_node(SyntaxKind::DESCRIPTION);
    let _g_string = p.start_node(SyntaxKind::STRING_VALUE);
    p.bump(SyntaxKind::STRING)
}

#[cfg(test)]
mod tests {
    use crate::ast;

    use super::*;
    #[test]
    fn it_can_access_arguments_in_fields() {
        let schema = r#"
"""
What a dumb description
"""
type Query {
  bestSellers(category: ProductCategory = ALL): [Product] @join__field(graph: PRODUCTS)
}
        "#;
        let parser = Parser::new(schema);
        let ast = parser.parse();

        assert!(ast.errors.is_empty());

        let document = ast.document();
        for definition in document.definitions() {
            if let ast::Definition::ObjectTypeDefinition(obj_def) = definition {
                assert_eq!(
                    obj_def
                        .description()
                        .unwrap()
                        .string_value()
                        .unwrap()
                        .to_string(),
                    "\"\"\"\nWhat a dumb description\n\"\"\"\n"
                );
                return;
            }
        }
        panic!("object type definition has not been catched");
    }
}
