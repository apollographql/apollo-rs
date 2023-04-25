use rowan::{GreenToken, SyntaxKind};

use crate::{ast, ast::AstNode, SyntaxNode, TokenText};
use std::num::{ParseFloatError, ParseIntError};

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
    pub fn text(self) -> Option<TokenText> {
        let txt = if self.query_token().is_some() {
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
        };

        txt.map(|txt| {
            TokenText(GreenToken::new(
                SyntaxKind(crate::SyntaxKind::DIRECTIVE_LOCATION as u16),
                txt,
            ))
        })
    }
}

impl ast::Definition {
    /// Return the name of this definition, if any. Schema definitions are unnamed and always
    /// return `None`.
    pub fn name(&self) -> Option<ast::Name> {
        match self {
            Self::OperationDefinition(it) => it.name(),
            Self::FragmentDefinition(it) => it.fragment_name()?.name(),
            Self::DirectiveDefinition(it) => it.name(),
            Self::SchemaDefinition(_) => None,
            Self::ScalarTypeDefinition(it) => it.name(),
            Self::ObjectTypeDefinition(it) => it.name(),
            Self::InterfaceTypeDefinition(it) => it.name(),
            Self::UnionTypeDefinition(it) => it.name(),
            Self::EnumTypeDefinition(it) => it.name(),
            Self::InputObjectTypeDefinition(it) => it.name(),
            Self::SchemaExtension(_) => None,
            Self::ScalarTypeExtension(it) => it.name(),
            Self::ObjectTypeExtension(it) => it.name(),
            Self::InterfaceTypeExtension(it) => it.name(),
            Self::UnionTypeExtension(it) => it.name(),
            Self::EnumTypeExtension(it) => it.name(),
            Self::InputObjectTypeExtension(it) => it.name(),
        }
    }

    pub fn kind(&self) -> &'static str {
        match self {
            ast::Definition::OperationDefinition(_) => "OperationDefinition",
            ast::Definition::FragmentDefinition(_) => "FragmentDefinition",
            ast::Definition::DirectiveDefinition(_) => "DirectiveDefinition",
            ast::Definition::ScalarTypeDefinition(_) => "ScalarTypeDefinition",
            ast::Definition::ObjectTypeDefinition(_) => "ObjectTypeDefinition",
            ast::Definition::InterfaceTypeDefinition(_) => "InterfaceTypeDefinition",
            ast::Definition::UnionTypeDefinition(_) => "UnionTypeDefinition",
            ast::Definition::EnumTypeDefinition(_) => "EnumTypeDefinition",
            ast::Definition::InputObjectTypeDefinition(_) => "InputObjectTypeDefinition",
            ast::Definition::SchemaDefinition(_) => "SchemaDefinition",
            ast::Definition::SchemaExtension(_) => "SchemaExtension",
            ast::Definition::ScalarTypeExtension(_) => "ScalarTypeExtension",
            ast::Definition::ObjectTypeExtension(_) => "ObjectTypeExtension",
            ast::Definition::InterfaceTypeExtension(_) => "InterfaceTypeExtension",
            ast::Definition::UnionTypeExtension(_) => "UnionTypeExtension",
            ast::Definition::EnumTypeExtension(_) => "EnumTypeExtension",
            ast::Definition::InputObjectTypeExtension(_) => "InputObjectTypeExtension",
        }
    }

    pub fn is_executable_definition(&self) -> bool {
        matches!(
            self,
            Self::OperationDefinition(_) | Self::FragmentDefinition(_)
        )
    }

    pub fn is_extension_definition(&self) -> bool {
        matches!(
            self,
            Self::SchemaExtension(_)
                | Self::ScalarTypeExtension(_)
                | Self::ObjectTypeExtension(_)
                | Self::InterfaceTypeExtension(_)
                | Self::UnionTypeExtension(_)
                | Self::EnumTypeExtension(_)
                | Self::InputObjectTypeExtension(_)
        )
    }
}

impl From<ast::StringValue> for String {
    fn from(val: ast::StringValue) -> Self {
        Self::from(&val)
    }
}

impl From<&'_ ast::StringValue> for String {
    fn from(val: &'_ ast::StringValue) -> Self {
        let text = text_of_first_token(val.syntax());
        text.trim_start_matches('"')
            .trim_end_matches('"')
            .to_string()
    }
}

impl TryFrom<ast::IntValue> for i32 {
    type Error = ParseIntError;

    fn try_from(val: ast::IntValue) -> Result<Self, Self::Error> {
        Self::try_from(&val)
    }
}

impl TryFrom<&'_ ast::IntValue> for i32 {
    type Error = ParseIntError;

    fn try_from(val: &'_ ast::IntValue) -> Result<Self, Self::Error> {
        let text = text_of_first_token(val.syntax());
        text.parse()
    }
}

impl TryFrom<ast::IntValue> for f64 {
    type Error = ParseFloatError;

    fn try_from(val: ast::IntValue) -> Result<Self, Self::Error> {
        Self::try_from(&val)
    }
}

impl TryFrom<&'_ ast::IntValue> for f64 {
    type Error = ParseFloatError;

    fn try_from(val: &'_ ast::IntValue) -> Result<Self, Self::Error> {
        let text = text_of_first_token(val.syntax());
        text.parse()
    }
}

impl TryFrom<ast::FloatValue> for f64 {
    type Error = ParseFloatError;

    fn try_from(val: ast::FloatValue) -> Result<Self, Self::Error> {
        Self::try_from(&val)
    }
}

impl TryFrom<&'_ ast::FloatValue> for f64 {
    type Error = ParseFloatError;

    fn try_from(val: &'_ ast::FloatValue) -> Result<Self, Self::Error> {
        let text = text_of_first_token(val.syntax());
        text.parse()
    }
}

impl TryFrom<ast::BooleanValue> for bool {
    type Error = std::str::ParseBoolError;

    fn try_from(val: ast::BooleanValue) -> Result<Self, Self::Error> {
        Self::try_from(&val)
    }
}

impl TryFrom<&'_ ast::BooleanValue> for bool {
    type Error = std::str::ParseBoolError;

    fn try_from(val: &'_ ast::BooleanValue) -> Result<Self, Self::Error> {
        let text = text_of_first_token(val.syntax());
        text.parse()
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
