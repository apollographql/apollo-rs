use crate::{ast, ast::AstNode, SyntaxNode, TokenText};
impl ast::Name {
    pub fn text(&self) -> TokenText {
        text_of_first_token(self.syntax())
    }
}

fn text_of_first_token(node: &SyntaxNode) -> TokenText {
    let first_token = node
        .green()
        .children()
        .next()
        .and_then(|it| it.into_token())
        .unwrap()
        .to_owned();

    TokenText(first_token)
}
