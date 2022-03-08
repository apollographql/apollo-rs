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

impl ast::DirectiveLocation {
    pub fn token_string(self) -> Option<&'static str> {
        if self.query_token().is_some() {
            Some("QUERY")
        } else if self.mutation_token().is_some() {
            Some("MUTATION")
        } else if self.subscription_token().is_some() {
            Some("SUBSCRIPTION")
        } else if self.field_token().is_some() {
            Some("FIELD")
        } else if self.fragment_definition_token().is_some() {
            Some("FRAGMENT_DEFINITION")
        } else if self.fragment_spread_token().is_some() {
            Some("FRAGMENT_SPREAD")
        } else if self.inline_fragment_token().is_some() {
            Some("INLINE_FRAGMENT")
        } else if self.variable_definition_token().is_some() {
            Some("VARIABLE_DEFINITION")
        } else if self.schema_token().is_some() {
            Some("SCHEMA")
        } else if self.scalar_token().is_some() {
            Some("SCALAR")
        } else if self.object_token().is_some() {
            Some("OBJECT")
        } else if self.field_definition_token().is_some() {
            Some("FIELD_DEFINITION")
        } else if self.argument_definition_token().is_some() {
            Some("ARGUMENT_DEFINITION")
        } else if self.interface_token().is_some() {
            Some("INTERFACE")
        } else if self.union_token().is_some() {
            Some("UNION")
        } else if self.enum_token().is_some() {
            Some("ENUM")
        } else if self.enum_value_token().is_some() {
            Some("ENUM_VALUE")
        } else if self.input_object_token().is_some() {
            Some("INPUT_OBJECT")
        } else if self.input_field_definition_token().is_some() {
            Some("INPUT_FIELD_DEFINITION")
        } else {
            None
        }
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

impl Into<i32> for ast::IntValue {
    fn into(self) -> i32 {
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
