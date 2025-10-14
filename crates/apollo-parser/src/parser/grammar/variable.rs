use crate::parser::grammar::description;
use crate::parser::grammar::directive;
use crate::parser::grammar::name;
use crate::parser::grammar::ty;
use crate::parser::grammar::value;
use crate::parser::grammar::value::Constness;
use crate::Parser;
use crate::SyntaxKind;
use crate::TokenKind;
use crate::S;
use crate::T;

/// See: https://spec.graphql.org/September2025/#sec-Language.Variables
///
/// *VariableDefinitions*:
///     **(** VariableDefinition* **)**
pub(crate) fn variable_definitions(p: &mut Parser) {
    let _g = p.start_node(SyntaxKind::VARIABLE_DEFINITIONS);
    p.bump(S!['(']);

    // Variable definitions can start with a description (string) or $ (variable)
    if let Some(TokenKind::StringValue | T![$]) = p.peek() {
        variable_definition(p);
    } else {
        p.err("expected a Variable Definition")
    }

    // Continue parsing while we see descriptions or variables
    while let Some(TokenKind::StringValue | T![$]) = p.peek() {
        variable_definition(p);
    }

    p.expect(T![')'], S![')']);
}

/// See: https://spec.graphql.org/September2025/#sec-Language.Variables
///
/// *VariableDefinition*:
///     Description? Variable **:** Type DefaultValue? Directives[Const]?
pub(crate) fn variable_definition(p: &mut Parser) {
    let _guard = p.start_node(SyntaxKind::VARIABLE_DEFINITION);

    // Check for optional description
    if let Some(TokenKind::StringValue) = p.peek() {
        description::description(p);
    }

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

/// See: https://spec.graphql.org/September2025/#sec-Language.Variables
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
    use crate::cst;
    use crate::Parser;

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
