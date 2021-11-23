use crate::{
    parser::grammar::{directive, name, ty, value},
    Parser, SyntaxKind, TokenKind, S, T,
};

/// See: https://spec.graphql.org/October2021/#VariableDefinitions
///
/// *VariableDefinitions*:
///     **(** VariableDefinition* **)**
pub(crate) fn variable_definitions(p: &mut Parser) {
    let _g = p.start_node(SyntaxKind::VARIABLE_DEFINITIONS);
    p.bump(S!['(']);

    // TODO @lrlna error: expected a variable definition to follow an opening brace
    if let Some(T![$]) = p.peek() {
        variable_definition(p, false);
    }
    p.expect(T![')'], S![')']);
}

/// See: https://spec.graphql.org/October2021/#VariableDefinition
///
/// *VariableDefinition*:
///     Variable **:** Type DefaultValue? Directives?
pub(crate) fn variable_definition(p: &mut Parser, is_variable: bool) {
    if let Some(T![$]) = p.peek() {
        let guard = p.start_node(SyntaxKind::VARIABLE_DEFINITION);
        variable(p);

        if let Some(T![:]) = p.peek() {
            p.bump(S![:]);
            if let Some(TokenKind::Name) = p.peek() {
                ty::ty(p);
                if let Some(T![=]) = p.peek() {
                    value::default_value(p);
                }
                if let Some(T![@]) = p.peek() {
                    directive::directives(p)
                }
                if p.peek().is_some() {
                    guard.finish_node();
                    return variable_definition(p, true);
                }
            }
            p.err("expected a Type");
        } else {
            p.err("expected a Name");
        }
    }

    if !is_variable {
        p.err("expected a Variable Definition");
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
