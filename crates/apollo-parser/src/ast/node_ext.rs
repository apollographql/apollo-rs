// This lint is here as we don't really need users to convert String/i64/f64
// into an AST Node. Should this change, we can remove this lint again.
#![allow(clippy::from_over_into)]

use crate::{ast, ast::AstNode, SyntaxNode, TokenText};

impl ast::Name {
    pub fn text(&self) -> TokenText {
        text_of_first_token(self.syntax())
    }
}

impl ast::Variable {
    pub fn text(&self) -> TokenText {
        self.name()
            .expect("Cannot get variable's NAME token")
            .text()
    }
}

impl ast::EnumValue {
    pub fn text(&self) -> TokenText {
        self.name()
            .expect("Cannot get enum value's NAME token")
            .text()
    }
}

impl Into<String> for ast::StringValue {
    fn into(self) -> String {
        let text = text_of_first_token(self.syntax());
        text.trim_start_matches('"')
            .trim_end_matches('"')
            .to_string()
    }
}

impl Into<i64> for ast::IntValue {
    fn into(self) -> i64 {
        let text = text_of_first_token(self.syntax());
        text.parse().expect("Cannot parse IntValue")
    }
}

impl Into<f64> for ast::FloatValue {
    fn into(self) -> f64 {
        let text = text_of_first_token(self.syntax());
        text.parse().expect("Cannot parse FloatValue")
    }
}

impl Into<bool> for ast::BooleanValue {
    fn into(self) -> bool {
        let text = text_of_first_token(self.syntax());
        text.parse().expect("Cannot parse BooleanValue")
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
