use crate::parser::grammar::value::Constness;
use crate::parser::grammar::{input, name, value};
use crate::{Parser, SyntaxKind, TokenKind, S, T};
use std::ops::ControlFlow;

/// See: https://spec.graphql.org/October2021/#Argument
///
/// *Argument[Const]*:
///    Name **:** Value[?Const]
pub(crate) fn argument(p: &mut Parser, constness: Constness) {
    let _guard = p.start_node(SyntaxKind::ARGUMENT);
    name::name(p);
    if let Some(T![:]) = p.peek() {
        p.bump(S![:]);
        value::value(p, constness, false);
    }
}

/// See: https://spec.graphql.org/October2021/#Arguments
///
/// *Arguments[Const]*:
///    **(** Argument[?Const]* **)**
pub(crate) fn arguments(p: &mut Parser, constness: Constness) {
    let _g = p.start_node(SyntaxKind::ARGUMENTS);
    p.bump(S!['(']);
    if let Some(TokenKind::Name) = p.peek() {
        argument(p, constness);
    } else {
        p.err("expected an Argument");
    }
    p.peek_while_kind(TokenKind::Name, |p| {
        argument(p, constness);
    });
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
    p.peek_while(|p, kind| match kind {
        TokenKind::Name | TokenKind::StringValue => {
            input::input_value_definition(p);
            ControlFlow::Continue(())
        }
        _ => ControlFlow::Break(()),
    });
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
