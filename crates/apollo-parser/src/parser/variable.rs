use crate::{
    create_err,
    parser::{name, ty},
    Parser, SyntaxKind, TokenKind,
};

/// See: https://spec.graphql.org/June2018/#VariableDefinition
///
/// ```txt
/// VariableDefinition
///     Variable : Type DefaultValue(opt)
/// ```
pub(crate) fn variable_definition(parser: &mut Parser, is_variable: bool) {
    // TODO @lrlna: parse optional default values
    if let Some(TokenKind::Dollar) = parser.peek() {
        let guard = parser.start_node(SyntaxKind::VARIABLE_DEFINITION);
        variable(parser);
        if let Some(TokenKind::Colon) = parser.peek() {
            parser.bump(SyntaxKind::COLON);
            if let Some(TokenKind::Node) = parser.peek() {
                // TODO @lrlna: type is a node, and needs its own parsing rules
                ty::ty(parser);
                if parser.peek().is_some() {
                    guard.finish_node();
                    return variable_definition(parser, true);
                }
            }
            parser.push_err(create_err!(
                parser.peek_data().unwrap(),
                "Expected Variable Definition to have a Type, got {}",
                parser.peek_data().unwrap()
            ));
        } else {
            parser.push_err(create_err!(
                parser.peek_data().unwrap(),
                "Expected Variable Definition to have a Name, got {}",
                parser.peek_data().unwrap()
            ));
        }
    }

    if let Some(TokenKind::Comma) = parser.peek() {
        parser.bump(SyntaxKind::COMMA);
        return variable_definition(parser, is_variable);
    }

    if !is_variable {
        parser.push_err(create_err!(
            parser.peek_data().unwrap(),
            "Expected to have an Variable Definition, got {}",
            parser.peek_data().unwrap()
        ));
    }
}

/// See: https://spec.graphql.org/June2018/#Variable
///
/// ```txt
/// Variable
///     $ Name
/// ```
pub(crate) fn variable(parser: &mut Parser) {
    let _guard = parser.start_node(SyntaxKind::VARIABLE);
    parser.bump(SyntaxKind::DOLLAR);
    name::name(parser);
}
