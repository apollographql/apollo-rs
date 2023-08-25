use rowan::{GreenToken, SyntaxKind};

use crate::{cst, cst::CstNode, SyntaxNode, TokenText};
use std::num::{ParseFloatError, ParseIntError};

impl cst::Name {
    pub fn text(&self) -> TokenText {
        text_of_first_token(self.syntax())
    }
}

impl cst::Variable {
    pub fn text(&self) -> TokenText {
        self.name()
            .expect("Cannot get variable's NAME token")
            .text()
    }
}

impl cst::EnumValue {
    pub fn text(&self) -> TokenText {
        self.name()
            .expect("Cannot get enum value's NAME token")
            .text()
    }
}

impl cst::DirectiveLocation {
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

impl cst::Definition {
    /// Return the name of this definition, if any. Schema definitions are unnamed and always
    /// return `None`.
    pub fn name(&self) -> Option<cst::Name> {
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
            cst::Definition::OperationDefinition(_) => "OperationDefinition",
            cst::Definition::FragmentDefinition(_) => "FragmentDefinition",
            cst::Definition::DirectiveDefinition(_) => "DirectiveDefinition",
            cst::Definition::ScalarTypeDefinition(_) => "ScalarTypeDefinition",
            cst::Definition::ObjectTypeDefinition(_) => "ObjectTypeDefinition",
            cst::Definition::InterfaceTypeDefinition(_) => "InterfaceTypeDefinition",
            cst::Definition::UnionTypeDefinition(_) => "UnionTypeDefinition",
            cst::Definition::EnumTypeDefinition(_) => "EnumTypeDefinition",
            cst::Definition::InputObjectTypeDefinition(_) => "InputObjectTypeDefinition",
            cst::Definition::SchemaDefinition(_) => "SchemaDefinition",
            cst::Definition::SchemaExtension(_) => "SchemaExtension",
            cst::Definition::ScalarTypeExtension(_) => "ScalarTypeExtension",
            cst::Definition::ObjectTypeExtension(_) => "ObjectTypeExtension",
            cst::Definition::InterfaceTypeExtension(_) => "InterfaceTypeExtension",
            cst::Definition::UnionTypeExtension(_) => "UnionTypeExtension",
            cst::Definition::EnumTypeExtension(_) => "EnumTypeExtension",
            cst::Definition::InputObjectTypeExtension(_) => "InputObjectTypeExtension",
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

impl From<cst::StringValue> for String {
    fn from(val: cst::StringValue) -> Self {
        Self::from(&val)
    }
}

/// Handle escaped characters in a StringValue. Panics on invalid escape sequences.
fn unescape_string(input: &str) -> String {
    let mut output = String::with_capacity(input.len());

    let mut iter = input.chars();
    while let Some(c) = iter.next() {
        match c {
            '\\' => {
                let Some(c2) = iter.next() else {
                    output.push(c);
                    break;
                };

                let mut unicode = || {
                    // 1. Let value be the 16-bit hexadecimal value represented
                    // by the sequence of hexadecimal digits within EscapedUnicode.
                    let value = iter.by_ref().take(4).fold(0, |acc, c| {
                        let digit = c.to_digit(16).unwrap();
                        (acc << 4) + digit
                    });
                    // 2. Return the code point value.
                    char::from_u32(value).unwrap()
                };

                match c2 {
                    'b' => output.push('\u{0008}'),
                    'f' => output.push('\u{000c}'),
                    'n' => output.push('\n'),
                    't' => output.push('\t'),
                    '"' | '\\' => output.push(c2),
                    'u' => output.push(unicode()),
                    _ => (),
                }
            }
            _ => output.push(c),
        }
    }

    output
}

impl From<&'_ cst::StringValue> for String {
    fn from(val: &'_ cst::StringValue) -> Self {
        let text = text_of_first_token(val.syntax());
        // Would panic if the contents are invalid, but the lexer already guarantees that the
        // string is valid.
        unescape_string(text.trim_start_matches('"').trim_end_matches('"'))
    }
}

impl TryFrom<cst::IntValue> for i32 {
    type Error = ParseIntError;

    fn try_from(val: cst::IntValue) -> Result<Self, Self::Error> {
        Self::try_from(&val)
    }
}

impl TryFrom<&'_ cst::IntValue> for i32 {
    type Error = ParseIntError;

    fn try_from(val: &'_ cst::IntValue) -> Result<Self, Self::Error> {
        let text = text_of_first_token(val.syntax());
        text.parse()
    }
}

impl TryFrom<cst::IntValue> for f64 {
    type Error = ParseFloatError;

    fn try_from(val: cst::IntValue) -> Result<Self, Self::Error> {
        Self::try_from(&val)
    }
}

impl TryFrom<&'_ cst::IntValue> for f64 {
    type Error = ParseFloatError;

    fn try_from(val: &'_ cst::IntValue) -> Result<Self, Self::Error> {
        let text = text_of_first_token(val.syntax());
        text.parse()
    }
}

impl TryFrom<cst::FloatValue> for f64 {
    type Error = ParseFloatError;

    fn try_from(val: cst::FloatValue) -> Result<Self, Self::Error> {
        Self::try_from(&val)
    }
}

impl TryFrom<&'_ cst::FloatValue> for f64 {
    type Error = ParseFloatError;

    fn try_from(val: &'_ cst::FloatValue) -> Result<Self, Self::Error> {
        let text = text_of_first_token(val.syntax());
        text.parse()
    }
}

impl TryFrom<cst::BooleanValue> for bool {
    type Error = std::str::ParseBoolError;

    fn try_from(val: cst::BooleanValue) -> Result<Self, Self::Error> {
        Self::try_from(&val)
    }
}

impl TryFrom<&'_ cst::BooleanValue> for bool {
    type Error = std::str::ParseBoolError;

    fn try_from(val: &'_ cst::BooleanValue) -> Result<Self, Self::Error> {
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
