use std::convert::TryFrom;

use apollo_parser::ast;
use thiserror::Error;

/// Errors that can occur when converting an apollo-parser AST to an apollo-encoder one.
#[derive(Debug, Clone, Error)]
pub enum FromError {
    #[error("parse tree is missing a node")]
    MissingNode,
    #[error("invalid i32")]
    ParseIntError(#[from] std::num::ParseIntError),
    #[error("invalid f64")]
    ParseFloatError(#[from] std::num::ParseFloatError),
}

impl TryFrom<ast::Value> for crate::Value {
    type Error = FromError;

    fn try_from(node: ast::Value) -> Result<Self, Self::Error> {
        let encoder_node = match node {
            ast::Value::Variable(variable) => Self::Variable(variable.name().ok_or(FromError::MissingNode)?.to_string()),
            ast::Value::StringValue(string) => Self::String(string.to_string()),
            ast::Value::FloatValue(float) => Self::Float(float.float_token().ok_or(FromError::MissingNode)?.text().parse()?),
            ast::Value::IntValue(int) => Self::Int(int.int_token().ok_or(FromError::MissingNode)?.text().parse()?),
            ast::Value::BooleanValue(boolean) => Self::Boolean(boolean.true_token().is_some()),
            ast::Value::NullValue(_) => Self::Null,
            ast::Value::EnumValue(enum_) => Self::Enum(enum_.to_string()),
            ast::Value::ListValue(list) => {
                let encoder_list = list.values()
                    .map(Self::try_from)
                    .collect::<Result<Vec<_>, FromError>>()?;
                Self::List(encoder_list)
            },
            ast::Value::ObjectValue(object) => {
                let encoder_object = object.object_fields()
                    .map(|field| {
                        let name = field.name().ok_or(FromError::MissingNode)?.to_string();
                        let value = field.value().ok_or(FromError::MissingNode)?.try_into()?;
                        Ok((name, value))
                    })
                    .collect::<Result<Vec<_>, FromError>>()?;
                Self::Object(encoder_object)
            },
        };

        Ok(encoder_node)
    }
}

impl TryFrom<ast::DefaultValue> for crate::Value {
    type Error = FromError;

    fn try_from(node: ast::DefaultValue) -> Result<Self, Self::Error> {
        node.value().ok_or(FromError::MissingNode)?.try_into()
    }
}

impl TryFrom<ast::Directive> for crate::Directive {
    type Error = FromError;

    fn try_from(node: ast::Directive) -> Result<Self, Self::Error> {
        let name = node.name().ok_or(FromError::MissingNode)?.to_string();
        let mut directive = Self::new(name);

        let arguments = node.arguments()
            .ok_or(FromError::MissingNode)?
            .arguments()
            .map(crate::Argument::try_from);
        for argument in arguments {
            directive.arg(argument?);
        }

        Ok(directive)
    }
}

impl TryFrom<ast::Argument> for crate::Argument {
    type Error = FromError;

    fn try_from(node: ast::Argument) -> Result<Self, Self::Error> {
        let name = node.name().ok_or(FromError::MissingNode)?.to_string();
        let value = node.value().ok_or(FromError::MissingNode)?.try_into()?;
        Ok(crate::Argument::new(name, value))
    }
}

impl TryFrom<ast::NamedType> for crate::Type_ {
    type Error = FromError;
    fn try_from(node: ast::NamedType) -> Result<Self, Self::Error> {
        Ok(Self::NamedType {
            name: node.name().ok_or(FromError::MissingNode)?.to_string(),
        })
    }
}

impl TryFrom<ast::ListType> for crate::Type_ {
    type Error = FromError;
    fn try_from(node: ast::ListType) -> Result<Self, Self::Error> {
        Ok(Self::List {
            ty: Box::new(node.ty().ok_or(FromError::MissingNode)?.try_into()?),
        })
    }
}

impl TryFrom<ast::NonNullType> for crate::Type_ {
    type Error = FromError;
    fn try_from(node: ast::NonNullType) -> Result<Self, Self::Error> {
        let named_type = node.named_type().ok_or(FromError::MissingNode).and_then(|ty| ty.try_into());
        let list_type = node.list_type().ok_or(FromError::MissingNode).and_then(|ty| ty.try_into());

        Ok(Self::NonNull {
            ty: Box::new(named_type.or(list_type)?),
        })
    }
}

impl TryFrom<ast::Type> for crate::Type_ {
    type Error = FromError;

    fn try_from(node: ast::Type) -> Result<Self, Self::Error> {
        match node {
            ast::Type::NamedType(ty) => ty.try_into(),
            ast::Type::ListType(ty) => ty.try_into(),
            ast::Type::NonNullType(ty) => ty.try_into(),
        }
    }
}

impl TryFrom<ast::InputValueDefinition> for crate::InputValueDefinition {
    type Error = FromError;

    fn try_from(node: ast::InputValueDefinition) -> Result<Self, Self::Error> {
        let name = node.name().ok_or(FromError::MissingNode)?.to_string();
        let ty = node.ty().ok_or(FromError::MissingNode)?;
        let mut encoder_node = Self::new(name, ty.try_into()?);
        if let Some(description) = node.description() {
            encoder_node.description(description.string_value().ok_or(FromError::MissingNode)?.to_string());
        }
        if let Some(default_value) = node.default_value() {
            // TODO represent this as a Value enum in encoder?
            encoder_node.default_value(default_value.value().ok_or(FromError::MissingNode)?.to_string());
        }
        Ok(encoder_node)
    }
}

impl TryFrom<ast::ArgumentsDefinition> for crate::ArgumentsDefinition {
    type Error = FromError;

    fn try_from(node: ast::ArgumentsDefinition) -> Result<Self, Self::Error> {
        let input_values = node.input_value_definitions()
            .map(|input_value| input_value.try_into())
            .collect::<Result<Vec<_>, FromError>>()?;

        Ok(Self::with_values(input_values))
    }
}

impl TryFrom<ast::FieldDefinition> for crate::FieldDefinition {
    type Error = FromError;

    fn try_from(node: ast::FieldDefinition) -> Result<Self, Self::Error> {
        let name = node.name().ok_or(FromError::MissingNode)?.to_string();
        let ty = node.ty().ok_or(FromError::MissingNode)?.try_into()?;
        let mut encoder_node = Self::new(name, ty);

        if let Some (arguments_definition) = node.arguments_definition() {
            for input_value in arguments_definition.input_value_definitions() {
                encoder_node.arg(input_value.try_into()?);
            }
        }

        if let Some(directives) = node.directives() {
            for directive in directives.directives() {
                encoder_node.directive(directive.try_into()?);
            }
        }

        Ok(encoder_node)
    }
}

impl TryFrom<ast::TypeCondition> for crate::TypeCondition {
    type Error = FromError;

    fn try_from(node: ast::TypeCondition) -> Result<Self, Self::Error> {
        let named_type = node.named_type().ok_or(FromError::MissingNode)?;
        let name = named_type.name().ok_or(FromError::MissingNode)?.to_string();
        Ok(Self::new(name))
    }
}

impl TryFrom<ast::Field> for crate::Field {
    type Error = FromError;

    fn try_from(node: ast::Field) -> Result<Self, Self::Error> {
        let name = node.name().ok_or(FromError::MissingNode)?.to_string();

        let mut encoder_node = Self::new(name);

        if let Some(alias) = node.alias() {
            let alias = alias.name().ok_or(FromError::MissingNode)?.to_string();
            encoder_node.alias(Some(alias));
        }

        let arguments = node.arguments()
            .ok_or(FromError::MissingNode)?
            .arguments()
            .map(crate::Argument::try_from);
        for argument in arguments {
            encoder_node.argument(argument?);
        }

        if let Some(directives) = node.directives() {
            for directive in directives.directives() {
                encoder_node.directive(directive.try_into()?);
            }
        }

        let selection_set = node.selection_set().map(|selection_set| selection_set.try_into()).transpose()?;
        encoder_node.selection_set(selection_set);

        Ok(encoder_node)
    }
}

impl TryFrom<ast::FragmentSpread> for crate::FragmentSpread {
    type Error = FromError;

    fn try_from(node: ast::FragmentSpread) -> Result<Self, Self::Error> {
        let name = node.fragment_name()
            .and_then(|fragment_name| fragment_name.name())
            .ok_or(FromError::MissingNode)?
            .to_string();
        let mut encoder_node = Self::new(name);
        if let Some(directives) = node.directives() {
            for directive in directives.directives() {
                encoder_node.directive(directive.try_into()?);
            }
        }
        Ok(encoder_node)
    }
}

impl TryFrom<ast::InlineFragment> for crate::InlineFragment {
    type Error = FromError;

    fn try_from(node: ast::InlineFragment) -> Result<Self, Self::Error> {
        let selection_set = node.selection_set()
            .ok_or(FromError::MissingNode)?
            .try_into()?;
        let mut encoder_node = Self::new(selection_set);

        let type_condition = node.type_condition()
            .map(|condition| condition.try_into())
            .transpose()?;
        encoder_node.type_condition(type_condition);

        if let Some(directives) = node.directives() {
            for directive in directives.directives() {
                encoder_node.directive(directive.try_into()?);
            }
        }

        Ok(encoder_node)
    }
}

impl TryFrom<ast::Selection> for crate::Selection {
    type Error = FromError;

    fn try_from(node: ast::Selection) -> Result<Self, Self::Error> {
        let encoder_node = match node {
            ast::Selection::Field(field) => Self::Field(field.try_into()?),
            ast::Selection::FragmentSpread(fragment) => Self::FragmentSpread(fragment.try_into()?),
            ast::Selection::InlineFragment(fragment) => Self::InlineFragment(fragment.try_into()?),
        };

        Ok(encoder_node)
    }
}

impl TryFrom<ast::SelectionSet> for crate::SelectionSet {
    type Error = FromError;

    fn try_from(node: ast::SelectionSet) -> Result<Self, Self::Error> {
        let selections = node.selections()
            .map(|selection| selection.try_into())
            .collect::<Result<Vec<_>, FromError>>()?;

        Ok(Self::with_selections(selections))
    }
}
