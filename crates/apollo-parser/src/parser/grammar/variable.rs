use crate::parser::grammar::value::Constness;
use crate::parser::grammar::{directive, name, ty, value};
use crate::{Parser, SyntaxKind, TokenKind, S, T};

/// See: https://spec.graphql.org/October2021/#VariableDefinitions
///
/// *VariableDefinitions*:
///     **(** VariableDefinition* **)**
pub(crate) fn variable_definitions(p: &mut Parser) {
    let _g = p.start_node(SyntaxKind::VARIABLE_DEFINITIONS);
    p.bump(S!['(']);

    if let Some(T![$]) = p.peek() {
        variable_definition(p);
    } else {
        p.err("expected a Variable Definition")
    }
    while let Some(T![$]) = p.peek() {
        variable_definition(p);
    }

    p.expect(T![')'], S![')']);
}

/// See: https://spec.graphql.org/October2021/#VariableDefinition
///
/// *VariableDefinition*:
///     Variable **:** Type DefaultValue? Directives[Const]?
pub(crate) fn variable_definition(p: &mut Parser) {
    let _guard = p.start_node(SyntaxKind::VARIABLE_DEFINITION);
    variable(p);

    if let Some(T![:]) = p.peek() {
        p.bump(S![:]);
        if let Some(TokenKind::Name | TokenKind::LBracket) = p.peek() {
            ty::ty(p);
            if let Some(T![=]) = p.peek() {
                value::default_value(p);
            }
            if let Some(T![@]) = p.peek() {
                directive::directives(p, Constness::Const)
            }
        } else {
            p.err("expected a Type");
        }
    } else {
        p.err("expected a Name");
    }
}

/// See: https://spec.graphql.org/October2021/#Variable
///
/// *Variable*:
///     **$** Name
pub(crate) fn variable(p: &mut Parser) {
    let _g = p.start_node(SyntaxKind::VARIABLE);
    p.bump(S![$]);
    name::name(p);
}

#[cfg(test)]

mod test {
    use crate::{cst, Parser};

    #[test]
    fn it_accesses_variable_name_and_type() {
        let gql = r#"
query GroceryStoreTrip($budget: Int) {
    name
}
        "#;

        let parser = Parser::new(gql);
        let cst = parser.parse();

        assert!(cst.errors().len() == 0);

        let doc = cst.document();

        for definition in doc.definitions() {
            if let cst::Definition::OperationDefinition(op_def) = definition {
                for var in op_def
                    .variable_definitions()
                    .unwrap()
                    .variable_definitions()
                {
                    assert_eq!(
                        var.variable().unwrap().name().unwrap().text().as_ref(),
                        "budget"
                    );
                    if let cst::Type::NamedType(name) = var.ty().unwrap() {
                        assert_eq!(name.name().unwrap().text().as_ref(), "Int")
                    }
                }
            }
        }
    }
}
