use std::convert::TryFrom;

use apollo_parser::ast::{self, AstNode};
use thiserror::Error;

/// Errors that can occur when converting an apollo-parser AST to an apollo-encoder one.
/// These errors don't give a lot of context at the moment. Before converting ASTs, you
/// should make sure that your parse tree is complete and valid.
// TODO(@goto-bus-stop) Would be nice to have some way to show where the error
// occurred
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

    /// Create an apollo-encoder node from an apollo-parser one.
    ///
    /// # Errors
    /// This returns an error if the apollo-parser tree is not valid. The error
    /// doesn't have much context due to TryFrom API constraints: validate the parse tree before
    /// using TryFrom if granular errors are important to you.
    fn try_from(node: ast::Value) -> Result<Self, Self::Error> {
        let encoder_node = match node {
            ast::Value::Variable(variable) => Self::Variable(
                variable
                    .name()
                    .ok_or(FromError::MissingNode)?
                    .text()
                    .to_string(),
            ),
            ast::Value::StringValue(string) => Self::String(string.into()),
            ast::Value::FloatValue(float) => Self::Float(
                float
                    .float_token()
                    .ok_or(FromError::MissingNode)?
                    .text()
                    .parse()?,
            ),
            ast::Value::IntValue(int) => Self::Int(
                int.int_token()
                    .ok_or(FromError::MissingNode)?
                    .text()
                    .parse()?,
            ),
            ast::Value::BooleanValue(boolean) => Self::Boolean(boolean.true_token().is_some()),
            ast::Value::NullValue(_) => Self::Null,
            ast::Value::EnumValue(enum_) => Self::Enum(enum_.text().to_string()),
            ast::Value::ListValue(list) => {
                let encoder_list = list
                    .values()
                    .map(Self::try_from)
                    .collect::<Result<Vec<_>, FromError>>()?;
                Self::List(encoder_list)
            }
            ast::Value::ObjectValue(object) => {
                let encoder_object = object
                    .object_fields()
                    .map(|field| {
                        let name = field
                            .name()
                            .ok_or(FromError::MissingNode)?
                            .text()
                            .to_string();
                        let value = field.value().ok_or(FromError::MissingNode)?.try_into()?;
                        Ok((name, value))
                    })
                    .collect::<Result<Vec<_>, FromError>>()?;
                Self::Object(encoder_object)
            }
        };

        Ok(encoder_node)
    }
}

impl TryFrom<ast::DefaultValue> for crate::Value {
    type Error = FromError;

    /// Create an apollo-encoder node from an apollo-parser one.
    ///
    /// # Errors
    /// This returns an error if the apollo-parser tree is not valid. The error
    /// doesn't have much context due to TryFrom API constraints: validate the parse tree before
    /// using TryFrom if granular errors are important to you.
    fn try_from(node: ast::DefaultValue) -> Result<Self, Self::Error> {
        node.value().ok_or(FromError::MissingNode)?.try_into()
    }
}

impl TryFrom<ast::Directive> for crate::Directive {
    type Error = FromError;

    /// Create an apollo-encoder node from an apollo-parser one.
    ///
    /// # Errors
    /// This returns an error if the apollo-parser tree is not valid. The error
    /// doesn't have much context due to TryFrom API constraints: validate the parse tree before
    /// using TryFrom if granular errors are important to you.
    fn try_from(node: ast::Directive) -> Result<Self, Self::Error> {
        let name = node
            .name()
            .ok_or(FromError::MissingNode)?
            .text()
            .to_string();
        let mut directive = Self::new(name);

        if let Some(arguments) = node.arguments() {
            for argument in arguments.arguments() {
                directive.arg(argument.try_into()?);
            }
        }

        Ok(directive)
    }
}

impl TryFrom<ast::Argument> for crate::Argument {
    type Error = FromError;

    /// Create an apollo-encoder node from an apollo-parser one.
    ///
    /// # Errors
    /// This returns an error if the apollo-parser tree is not valid. The error
    /// doesn't have much context due to TryFrom API constraints: validate the parse tree before
    /// using TryFrom if granular errors are important to you.
    fn try_from(node: ast::Argument) -> Result<Self, Self::Error> {
        let name = node
            .name()
            .ok_or(FromError::MissingNode)?
            .text()
            .to_string();
        let value = node.value().ok_or(FromError::MissingNode)?.try_into()?;
        Ok(crate::Argument::new(name, value))
    }
}

impl TryFrom<ast::NamedType> for crate::Type_ {
    type Error = FromError;

    /// Create an apollo-encoder node from an apollo-parser one.
    ///
    /// # Errors
    /// This returns an error if the apollo-parser tree is not valid. The error
    /// doesn't have much context due to TryFrom API constraints: validate the parse tree before
    /// using TryFrom if granular errors are important to you.
    fn try_from(node: ast::NamedType) -> Result<Self, Self::Error> {
        Ok(Self::NamedType {
            name: node
                .name()
                .ok_or(FromError::MissingNode)?
                .text()
                .to_string(),
        })
    }
}

impl TryFrom<ast::ListType> for crate::Type_ {
    type Error = FromError;

    /// Create an apollo-encoder node from an apollo-parser one.
    ///
    /// # Errors
    /// This returns an error if the apollo-parser tree is not valid. The error
    /// doesn't have much context due to TryFrom API constraints: validate the parse tree before
    /// using TryFrom if granular errors are important to you.
    fn try_from(node: ast::ListType) -> Result<Self, Self::Error> {
        Ok(Self::List {
            ty: Box::new(node.ty().ok_or(FromError::MissingNode)?.try_into()?),
        })
    }
}

impl TryFrom<ast::NonNullType> for crate::Type_ {
    type Error = FromError;

    /// Create an apollo-encoder node from an apollo-parser one.
    ///
    /// # Errors
    /// This returns an error if the apollo-parser tree is not valid. The error
    /// doesn't have much context due to TryFrom API constraints: validate the parse tree before
    /// using TryFrom if granular errors are important to you.
    fn try_from(node: ast::NonNullType) -> Result<Self, Self::Error> {
        let named_type = node
            .named_type()
            .ok_or(FromError::MissingNode)
            .and_then(|ty| ty.try_into());
        let list_type = node
            .list_type()
            .ok_or(FromError::MissingNode)
            .and_then(|ty| ty.try_into());

        Ok(Self::NonNull {
            ty: Box::new(named_type.or(list_type)?),
        })
    }
}

impl TryFrom<ast::Type> for crate::Type_ {
    type Error = FromError;

    /// Create an apollo-encoder node from an apollo-parser one.
    ///
    /// # Errors
    /// This returns an error if the apollo-parser tree is not valid. The error
    /// doesn't have much context due to TryFrom API constraints: validate the parse tree before
    /// using TryFrom if granular errors are important to you.
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

    /// Create an apollo-encoder node from an apollo-parser one.
    ///
    /// # Errors
    /// This returns an error if the apollo-parser tree is not valid. The error
    /// doesn't have much context due to TryFrom API constraints: validate the parse tree before
    /// using TryFrom if granular errors are important to you.
    fn try_from(node: ast::InputValueDefinition) -> Result<Self, Self::Error> {
        let name = node
            .name()
            .ok_or(FromError::MissingNode)?
            .text()
            .to_string();
        let ty = node.ty().ok_or(FromError::MissingNode)?;
        let mut encoder_node = Self::new(name, ty.try_into()?);

        if let Some(description) = node.description() {
            encoder_node.description(
                description
                    .string_value()
                    .ok_or(FromError::MissingNode)?
                    .into(),
            );
        }

        if let Some(default_value) = node.default_value() {
            // TODO represent this as a Value enum in encoder?
            encoder_node.default_value(
                default_value
                    .value()
                    .ok_or(FromError::MissingNode)?
                    .source_string(),
            );
        }

        if let Some(directives) = node.directives() {
            for directive in directives.directives() {
                encoder_node.directive(directive.try_into()?);
            }
        }

        Ok(encoder_node)
    }
}

impl TryFrom<ast::ArgumentsDefinition> for crate::ArgumentsDefinition {
    type Error = FromError;

    /// Create an apollo-encoder node from an apollo-parser one.
    ///
    /// # Errors
    /// This returns an error if the apollo-parser tree is not valid. The error
    /// doesn't have much context due to TryFrom API constraints: validate the parse tree before
    /// using TryFrom if granular errors are important to you.
    fn try_from(node: ast::ArgumentsDefinition) -> Result<Self, Self::Error> {
        let input_values = node
            .input_value_definitions()
            .map(|input_value| input_value.try_into())
            .collect::<Result<Vec<_>, FromError>>()?;

        Ok(Self::with_values(input_values))
    }
}

impl TryFrom<ast::FieldDefinition> for crate::FieldDefinition {
    type Error = FromError;

    /// Create an apollo-encoder node from an apollo-parser one.
    ///
    /// # Errors
    /// This returns an error if the apollo-parser tree is not valid. The error
    /// doesn't have much context due to TryFrom API constraints: validate the parse tree before
    /// using TryFrom if granular errors are important to you.
    fn try_from(node: ast::FieldDefinition) -> Result<Self, Self::Error> {
        let name = node
            .name()
            .ok_or(FromError::MissingNode)?
            .text()
            .to_string();
        let ty = node.ty().ok_or(FromError::MissingNode)?.try_into()?;
        let mut encoder_node = Self::new(name, ty);

        if let Some(arguments_definition) = node.arguments_definition() {
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

    /// Create an apollo-encoder node from an apollo-parser one.
    ///
    /// # Errors
    /// This returns an error if the apollo-parser tree is not valid. The error
    /// doesn't have much context due to TryFrom API constraints: validate the parse tree before
    /// using TryFrom if granular errors are important to you.
    fn try_from(node: ast::TypeCondition) -> Result<Self, Self::Error> {
        let named_type = node.named_type().ok_or(FromError::MissingNode)?;
        let name = named_type
            .name()
            .ok_or(FromError::MissingNode)?
            .text()
            .to_string();
        Ok(Self::new(name))
    }
}

impl TryFrom<ast::Field> for crate::Field {
    type Error = FromError;

    /// Create an apollo-encoder node from an apollo-parser one.
    ///
    /// # Errors
    /// This returns an error if the apollo-parser tree is not valid. The error
    /// doesn't have much context due to TryFrom API constraints: validate the parse tree before
    /// using TryFrom if granular errors are important to you.
    fn try_from(node: ast::Field) -> Result<Self, Self::Error> {
        let name = node
            .name()
            .ok_or(FromError::MissingNode)?
            .text()
            .to_string();

        let mut encoder_node = Self::new(name);

        if let Some(alias) = node.alias() {
            let alias = alias
                .name()
                .ok_or(FromError::MissingNode)?
                .text()
                .to_string();
            encoder_node.alias(Some(alias));
        }

        if let Some(arguments) = node.arguments() {
            for argument in arguments.arguments() {
                encoder_node.argument(argument.try_into()?);
            }
        }

        if let Some(directives) = node.directives() {
            for directive in directives.directives() {
                encoder_node.directive(directive.try_into()?);
            }
        }

        let selection_set = node
            .selection_set()
            .map(|selection_set| selection_set.try_into())
            .transpose()?;
        encoder_node.selection_set(selection_set);

        Ok(encoder_node)
    }
}

impl TryFrom<ast::FragmentSpread> for crate::FragmentSpread {
    type Error = FromError;

    /// Create an apollo-encoder node from an apollo-parser one.
    ///
    /// # Errors
    /// This returns an error if the apollo-parser tree is not valid. The error
    /// doesn't have much context due to TryFrom API constraints: validate the parse tree before
    /// using TryFrom if granular errors are important to you.
    fn try_from(node: ast::FragmentSpread) -> Result<Self, Self::Error> {
        let name = node
            .fragment_name()
            .and_then(|fragment_name| fragment_name.name())
            .ok_or(FromError::MissingNode)?
            .text()
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

    /// Create an apollo-encoder node from an apollo-parser one.
    ///
    /// # Errors
    /// This returns an error if the apollo-parser tree is not valid. The error
    /// doesn't have much context due to TryFrom API constraints: validate the parse tree before
    /// using TryFrom if granular errors are important to you.
    fn try_from(node: ast::InlineFragment) -> Result<Self, Self::Error> {
        let selection_set = node
            .selection_set()
            .ok_or(FromError::MissingNode)?
            .try_into()?;
        let mut encoder_node = Self::new(selection_set);

        let type_condition = node
            .type_condition()
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

    /// Create an apollo-encoder node from an apollo-parser one.
    ///
    /// # Errors
    /// This returns an error if the apollo-parser tree is not valid. The error
    /// doesn't have much context due to TryFrom API constraints: validate the parse tree before
    /// using TryFrom if granular errors are important to you.
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

    /// Create an apollo-encoder node from an apollo-parser one.
    ///
    /// # Errors
    /// This returns an error if the apollo-parser tree is not valid. The error
    /// doesn't have much context due to TryFrom API constraints: validate the parse tree before
    /// using TryFrom if granular errors are important to you.
    fn try_from(node: ast::SelectionSet) -> Result<Self, Self::Error> {
        let selections = node
            .selections()
            .map(|selection| selection.try_into())
            .collect::<Result<Vec<_>, FromError>>()?;

        Ok(Self::with_selections(selections))
    }
}

impl TryFrom<ast::OperationType> for crate::OperationType {
    type Error = FromError;

    /// Create an apollo-encoder node from an apollo-parser one.
    ///
    /// # Errors
    /// This returns an error if the apollo-parser tree is not valid. The error
    /// doesn't have much context due to TryFrom API constraints: validate the parse tree before
    /// using TryFrom if granular errors are important to you.
    fn try_from(node: ast::OperationType) -> Result<Self, Self::Error> {
        if node.query_token().is_some() {
            Ok(Self::Query)
        } else if node.mutation_token().is_some() {
            Ok(Self::Mutation)
        } else if node.subscription_token().is_some() {
            Ok(Self::Subscription)
        } else {
            Err(FromError::MissingNode)
        }
    }
}

impl TryFrom<ast::VariableDefinition> for crate::VariableDefinition {
    type Error = FromError;

    /// Create an apollo-encoder node from an apollo-parser one.
    ///
    /// # Errors
    /// This returns an error if the apollo-parser tree is not valid. The error
    /// doesn't have much context due to TryFrom API constraints: validate the parse tree before
    /// using TryFrom if granular errors are important to you.
    fn try_from(node: ast::VariableDefinition) -> Result<Self, Self::Error> {
        let name = node
            .variable()
            .ok_or(FromError::MissingNode)?
            .name()
            .ok_or(FromError::MissingNode)?
            .text()
            .to_string();
        let ty = node.ty().ok_or(FromError::MissingNode)?.try_into()?;

        let mut encoder_node = Self::new(name, ty);

        if let Some(default_value) = node.default_value() {
            encoder_node.default_value(default_value.try_into()?);
        }

        if let Some(directives) = node.directives() {
            for directive in directives.directives() {
                encoder_node.directive(directive.try_into()?);
            }
        }

        Ok(encoder_node)
    }
}

impl TryFrom<ast::OperationDefinition> for crate::OperationDefinition {
    type Error = FromError;

    /// Create an apollo-encoder node from an apollo-parser one.
    ///
    /// # Errors
    /// This returns an error if the apollo-parser tree is not valid. The error
    /// doesn't have much context due to TryFrom API constraints: validate the parse tree before
    /// using TryFrom if granular errors are important to you.
    fn try_from(node: ast::OperationDefinition) -> Result<Self, Self::Error> {
        let operation_type = node
            .operation_type()
            .ok_or(FromError::MissingNode)?
            .try_into()?;
        let selection_set = node
            .selection_set()
            .ok_or(FromError::MissingNode)?
            .try_into()?;

        let mut encoder_node = Self::new(operation_type, selection_set);

        if let Some(name) = node.name() {
            encoder_node.name(Some(name.text().to_string()));
        }

        if let Some(variable_definitions) = node.variable_definitions() {
            for variable_definition in variable_definitions.variable_definitions() {
                encoder_node.variable_definition(variable_definition.try_into()?);
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

impl TryFrom<ast::FragmentDefinition> for crate::FragmentDefinition {
    type Error = FromError;

    /// Create an apollo-encoder node from an apollo-parser one.
    ///
    /// # Errors
    /// This returns an error if the apollo-parser tree is not valid. The error
    /// doesn't have much context due to TryFrom API constraints: validate the parse tree before
    /// using TryFrom if granular errors are important to you.
    fn try_from(node: ast::FragmentDefinition) -> Result<Self, Self::Error> {
        let name = node
            .fragment_name()
            .ok_or(FromError::MissingNode)?
            .name()
            .ok_or(FromError::MissingNode)?
            .text()
            .to_string();
        let type_condition = node
            .type_condition()
            .ok_or(FromError::MissingNode)?
            .try_into()?;
        let selection_set = node
            .selection_set()
            .ok_or(FromError::MissingNode)?
            .try_into()?;

        let mut encoder_node = Self::new(name, type_condition, selection_set);
        if let Some(directives) = node.directives() {
            for directive in directives.directives() {
                encoder_node.directive(directive.try_into()?);
            }
        }

        Ok(encoder_node)
    }
}

impl TryFrom<ast::DirectiveDefinition> for crate::DirectiveDefinition {
    type Error = FromError;

    /// Create an apollo-encoder node from an apollo-parser one.
    ///
    /// # Errors
    /// This returns an error if the apollo-parser tree is not valid. The error
    /// doesn't have much context due to TryFrom API constraints: validate the parse tree before
    /// using TryFrom if granular errors are important to you.
    fn try_from(node: ast::DirectiveDefinition) -> Result<Self, Self::Error> {
        let name = node
            .name()
            .ok_or(FromError::MissingNode)?
            .text()
            .to_string();

        let mut encoder_node = Self::new(name);

        let description = node
            .description()
            .and_then(|description| description.string_value())
            .map(|string| string.into());
        if let Some(description) = description {
            encoder_node.description(description);
        }

        if let Some(arguments_definition) = node.arguments_definition() {
            for input_value in arguments_definition.input_value_definitions() {
                encoder_node.arg(input_value.try_into()?);
            }
        }

        if let Some(directive_locations) = node.directive_locations() {
            let locations = directive_locations
                .directive_locations()
                .map(|location| location.text().map(|token| token.to_string()));
            for location in locations {
                // TODO(@goto-bus-stop) This actually indicates that a directive location had an
                // unknown value, not that it was missing
                let location = location.ok_or(FromError::MissingNode)?;
                encoder_node.location(location);
            }
        }

        if node.repeatable_token().is_some() {
            encoder_node.repeatable();
        }

        Ok(encoder_node)
    }
}

fn apply_root_operation_type_definitions(
    encoder_node: &mut crate::SchemaDefinition,
    type_definitions: impl Iterator<Item = ast::RootOperationTypeDefinition>,
) -> Result<(), FromError> {
    for root in type_definitions {
        let operation_type = root.operation_type().ok_or(FromError::MissingNode)?;
        let name = root
            .named_type()
            .ok_or(FromError::MissingNode)?
            .name()
            .ok_or(FromError::MissingNode)?
            .text()
            .to_string();
        if operation_type.query_token().is_some() {
            encoder_node.query(name);
        } else if operation_type.mutation_token().is_some() {
            encoder_node.mutation(name);
        } else if operation_type.subscription_token().is_some() {
            encoder_node.subscription(name);
        } else {
            return Err(FromError::MissingNode);
        }
    }

    Ok(())
}

impl TryFrom<ast::SchemaDefinition> for crate::SchemaDefinition {
    type Error = FromError;

    /// Create an apollo-encoder node from an apollo-parser one.
    ///
    /// # Errors
    /// This returns an error if the apollo-parser tree is not valid. The error
    /// doesn't have much context due to TryFrom API constraints: validate the parse tree before
    /// using TryFrom if granular errors are important to you.
    fn try_from(node: ast::SchemaDefinition) -> Result<Self, Self::Error> {
        let mut encoder_node = Self::new();

        let description = node
            .description()
            .and_then(|description| description.string_value())
            .map(|string| string.into());
        if let Some(description) = description {
            encoder_node.description(description);
        }

        if let Some(directives) = node.directives() {
            for directive in directives.directives() {
                encoder_node.directive(directive.try_into()?);
            }
        }

        apply_root_operation_type_definitions(
            &mut encoder_node,
            node.root_operation_type_definitions(),
        )?;

        Ok(encoder_node)
    }
}

impl TryFrom<ast::ScalarTypeDefinition> for crate::ScalarDefinition {
    type Error = FromError;

    /// Create an apollo-encoder node from an apollo-parser one.
    ///
    /// # Errors
    /// This returns an error if the apollo-parser tree is not valid. The error
    /// doesn't have much context due to TryFrom API constraints: validate the parse tree before
    /// using TryFrom if granular errors are important to you.
    fn try_from(node: ast::ScalarTypeDefinition) -> Result<Self, Self::Error> {
        let name = node
            .name()
            .ok_or(FromError::MissingNode)?
            .text()
            .to_string();
        let mut encoder_node = Self::new(name);

        let description = node
            .description()
            .and_then(|description| description.string_value())
            .map(|string| string.into());
        if let Some(description) = description {
            encoder_node.description(description);
        }

        if let Some(directives) = node.directives() {
            for directive in directives.directives() {
                encoder_node.directive(directive.try_into()?);
            }
        }

        Ok(encoder_node)
    }
}

impl TryFrom<ast::ObjectTypeDefinition> for crate::ObjectDefinition {
    type Error = FromError;

    /// Create an apollo-encoder node from an apollo-parser one.
    ///
    /// # Errors
    /// This returns an error if the apollo-parser tree is not valid. The error
    /// doesn't have much context due to TryFrom API constraints: validate the parse tree before
    /// using TryFrom if granular errors are important to you.
    fn try_from(node: ast::ObjectTypeDefinition) -> Result<Self, Self::Error> {
        let name = node
            .name()
            .ok_or(FromError::MissingNode)?
            .text()
            .to_string();
        let mut encoder_node = Self::new(name);

        let description = node
            .description()
            .and_then(|description| description.string_value())
            .map(|string| string.into());
        if let Some(description) = description {
            encoder_node.description(description);
        }

        if let Some(directives) = node.directives() {
            for directive in directives.directives() {
                encoder_node.directive(directive.try_into()?);
            }
        }

        if let Some(implements_interfaces) = node.implements_interfaces() {
            for implements in implements_interfaces.named_types() {
                let name = implements
                    .name()
                    .ok_or(FromError::MissingNode)?
                    .text()
                    .to_string();
                encoder_node.interface(name);
            }
        }

        if let Some(field_definitions) = node.fields_definition() {
            for field_definition in field_definitions.field_definitions() {
                encoder_node.field(field_definition.try_into()?);
            }
        }

        Ok(encoder_node)
    }
}

impl TryFrom<ast::InterfaceTypeDefinition> for crate::InterfaceDefinition {
    type Error = FromError;

    /// Create an apollo-encoder node from an apollo-parser one.
    ///
    /// # Errors
    /// This returns an error if the apollo-parser tree is not valid. The error
    /// doesn't have much context due to TryFrom API constraints: validate the parse tree before
    /// using TryFrom if granular errors are important to you.
    fn try_from(node: ast::InterfaceTypeDefinition) -> Result<Self, Self::Error> {
        let name = node
            .name()
            .ok_or(FromError::MissingNode)?
            .text()
            .to_string();
        let mut encoder_node = Self::new(name);

        let description = node
            .description()
            .and_then(|description| description.string_value())
            .map(|string| string.into());
        if let Some(description) = description {
            encoder_node.description(description);
        }

        if let Some(directives) = node.directives() {
            for directive in directives.directives() {
                encoder_node.directive(directive.try_into()?);
            }
        }

        if let Some(implements_interfaces) = node.implements_interfaces() {
            for implements in implements_interfaces.named_types() {
                let name = implements
                    .name()
                    .ok_or(FromError::MissingNode)?
                    .text()
                    .to_string();
                encoder_node.interface(name);
            }
        }

        if let Some(field_definitions) = node.fields_definition() {
            for field_definition in field_definitions.field_definitions() {
                encoder_node.field(field_definition.try_into()?);
            }
        }

        Ok(encoder_node)
    }
}

impl TryFrom<ast::UnionTypeDefinition> for crate::UnionDefinition {
    type Error = FromError;

    /// Create an apollo-encoder node from an apollo-parser one.
    ///
    /// # Errors
    /// This returns an error if the apollo-parser tree is not valid. The error
    /// doesn't have much context due to TryFrom API constraints: validate the parse tree before
    /// using TryFrom if granular errors are important to you.
    fn try_from(node: ast::UnionTypeDefinition) -> Result<Self, Self::Error> {
        let name = node
            .name()
            .ok_or(FromError::MissingNode)?
            .text()
            .to_string();
        let mut encoder_node = Self::new(name);

        let description = node
            .description()
            .and_then(|description| description.string_value())
            .map(|string| string.into());
        if let Some(description) = description {
            encoder_node.description(description);
        }

        if let Some(directives) = node.directives() {
            for directive in directives.directives() {
                encoder_node.directive(directive.try_into()?);
            }
        }

        if let Some(members) = node.union_member_types() {
            for member in members.named_types() {
                encoder_node.member(
                    member
                        .name()
                        .ok_or(FromError::MissingNode)?
                        .text()
                        .to_string(),
                );
            }
        }

        Ok(encoder_node)
    }
}

impl TryFrom<ast::EnumValueDefinition> for crate::EnumValue {
    type Error = FromError;

    /// Create an apollo-encoder node from an apollo-parser one.
    ///
    /// # Errors
    /// This returns an error if the apollo-parser tree is not valid. The error
    /// doesn't have much context due to TryFrom API constraints: validate the parse tree before
    /// using TryFrom if granular errors are important to you.
    fn try_from(node: ast::EnumValueDefinition) -> Result<Self, Self::Error> {
        let name = node
            .enum_value()
            .ok_or(FromError::MissingNode)?
            .name()
            .ok_or(FromError::MissingNode)?
            .text()
            .to_string();
        let mut encoder_node = Self::new(name);

        let description = node
            .description()
            .and_then(|description| description.string_value())
            .map(|string| string.into());
        if let Some(description) = description {
            encoder_node.description(description);
        }

        if let Some(directives) = node.directives() {
            for directive in directives.directives() {
                encoder_node.directive(directive.try_into()?);
            }
        }

        Ok(encoder_node)
    }
}

impl TryFrom<ast::EnumTypeDefinition> for crate::EnumDefinition {
    type Error = FromError;

    /// Create an apollo-encoder node from an apollo-parser one.
    ///
    /// # Errors
    /// This returns an error if the apollo-parser tree is not valid. The error
    /// doesn't have much context due to TryFrom API constraints: validate the parse tree before
    /// using TryFrom if granular errors are important to you.
    fn try_from(node: ast::EnumTypeDefinition) -> Result<Self, Self::Error> {
        let name = node
            .name()
            .ok_or(FromError::MissingNode)?
            .text()
            .to_string();
        let mut encoder_node = Self::new(name);

        let description = node
            .description()
            .and_then(|description| description.string_value())
            .map(|string| string.into());
        if let Some(description) = description {
            encoder_node.description(description);
        }

        if let Some(directives) = node.directives() {
            for directive in directives.directives() {
                encoder_node.directive(directive.try_into()?);
            }
        }

        if let Some(values) = node.enum_values_definition() {
            for value in values.enum_value_definitions() {
                encoder_node.value(value.try_into()?);
            }
        }

        Ok(encoder_node)
    }
}

impl TryFrom<ast::InputValueDefinition> for crate::InputField {
    type Error = FromError;

    /// Create an apollo-encoder node from an apollo-parser one.
    ///
    /// # Errors
    /// This returns an error if the apollo-parser tree is not valid. The error
    /// doesn't have much context due to TryFrom API constraints: validate the parse tree before
    /// using TryFrom if granular errors are important to you.
    fn try_from(node: ast::InputValueDefinition) -> Result<Self, Self::Error> {
        let name = node
            .name()
            .ok_or(FromError::MissingNode)?
            .text()
            .to_string();
        let ty = node.ty().ok_or(FromError::MissingNode)?;
        let mut encoder_node = Self::new(name, ty.try_into()?);

        if let Some(description) = node.description() {
            encoder_node.description(
                description
                    .string_value()
                    .ok_or(FromError::MissingNode)?
                    .into(),
            );
        }

        if let Some(default_value) = node.default_value() {
            // TODO represent this as a Value enum in encoder?
            encoder_node.default_value(
                default_value
                    .value()
                    .ok_or(FromError::MissingNode)?
                    .source_string(),
            );
        }

        if let Some(directives) = node.directives() {
            for directive in directives.directives() {
                encoder_node.directive(directive.try_into()?);
            }
        }

        Ok(encoder_node)
    }
}

impl TryFrom<ast::InputObjectTypeDefinition> for crate::InputObjectDefinition {
    type Error = FromError;

    /// Create an apollo-encoder node from an apollo-parser one.
    ///
    /// # Errors
    /// This returns an error if the apollo-parser tree is not valid. The error
    /// doesn't have much context due to TryFrom API constraints: validate the parse tree before
    /// using TryFrom if granular errors are important to you.
    fn try_from(node: ast::InputObjectTypeDefinition) -> Result<Self, Self::Error> {
        let name = node
            .name()
            .ok_or(FromError::MissingNode)?
            .text()
            .to_string();
        let mut encoder_node = Self::new(name);

        let description = node
            .description()
            .and_then(|description| description.string_value())
            .map(|string| string.into());
        if let Some(description) = description {
            encoder_node.description(description);
        }

        if let Some(directives) = node.directives() {
            for directive in directives.directives() {
                encoder_node.directive(directive.try_into()?);
            }
        }

        if let Some(field_definitions) = node.input_fields_definition() {
            for field_definition in field_definitions.input_value_definitions() {
                encoder_node.field(field_definition.try_into()?);
            }
        }

        Ok(encoder_node)
    }
}

impl TryFrom<ast::SchemaExtension> for crate::SchemaDefinition {
    type Error = FromError;

    /// Create an apollo-encoder node from an apollo-parser one.
    ///
    /// # Errors
    /// This returns an error if the apollo-parser tree is not valid. The error
    /// doesn't have much context due to TryFrom API constraints: validate the parse tree before
    /// using TryFrom if granular errors are important to you.
    fn try_from(node: ast::SchemaExtension) -> Result<Self, Self::Error> {
        let mut encoder_node = Self::new();

        if let Some(directives) = node.directives() {
            for directive in directives.directives() {
                encoder_node.directive(directive.try_into()?);
            }
        }

        apply_root_operation_type_definitions(
            &mut encoder_node,
            node.root_operation_type_definitions(),
        )?;

        encoder_node.extend();

        Ok(encoder_node)
    }
}

impl TryFrom<ast::ScalarTypeExtension> for crate::ScalarDefinition {
    type Error = FromError;

    /// Create an apollo-encoder node from an apollo-parser one.
    ///
    /// # Errors
    /// This returns an error if the apollo-parser tree is not valid. The error
    /// doesn't have much context due to TryFrom API constraints: validate the parse tree before
    /// using TryFrom if granular errors are important to you.
    fn try_from(node: ast::ScalarTypeExtension) -> Result<Self, Self::Error> {
        let name = node
            .name()
            .ok_or(FromError::MissingNode)?
            .text()
            .to_string();
        let mut encoder_node = Self::new(name);

        if let Some(directives) = node.directives() {
            for directive in directives.directives() {
                encoder_node.directive(directive.try_into()?);
            }
        }

        encoder_node.extend();

        Ok(encoder_node)
    }
}

impl TryFrom<ast::ObjectTypeExtension> for crate::ObjectDefinition {
    type Error = FromError;

    /// Create an apollo-encoder node from an apollo-parser one.
    ///
    /// # Errors
    /// This returns an error if the apollo-parser tree is not valid. The error
    /// doesn't have much context due to TryFrom API constraints: validate the parse tree before
    /// using TryFrom if granular errors are important to you.
    fn try_from(node: ast::ObjectTypeExtension) -> Result<Self, Self::Error> {
        let name = node
            .name()
            .ok_or(FromError::MissingNode)?
            .text()
            .to_string();
        let mut encoder_node = Self::new(name);

        if let Some(directives) = node.directives() {
            for directive in directives.directives() {
                encoder_node.directive(directive.try_into()?);
            }
        }

        if let Some(implements_interfaces) = node.implements_interfaces() {
            for implements in implements_interfaces.named_types() {
                let name = implements
                    .name()
                    .ok_or(FromError::MissingNode)?
                    .text()
                    .to_string();
                encoder_node.interface(name);
            }
        }

        if let Some(field_definitions) = node.fields_definition() {
            for field_definition in field_definitions.field_definitions() {
                encoder_node.field(field_definition.try_into()?);
            }
        }

        encoder_node.extend();

        Ok(encoder_node)
    }
}

impl TryFrom<ast::InterfaceTypeExtension> for crate::InterfaceDefinition {
    type Error = FromError;

    /// Create an apollo-encoder node from an apollo-parser one.
    ///
    /// # Errors
    /// This returns an error if the apollo-parser tree is not valid. The error
    /// doesn't have much context due to TryFrom API constraints: validate the parse tree before
    /// using TryFrom if granular errors are important to you.
    fn try_from(node: ast::InterfaceTypeExtension) -> Result<Self, Self::Error> {
        let name = node
            .name()
            .ok_or(FromError::MissingNode)?
            .text()
            .to_string();
        let mut encoder_node = Self::new(name);

        if let Some(directives) = node.directives() {
            for directive in directives.directives() {
                encoder_node.directive(directive.try_into()?);
            }
        }

        if let Some(implements_interfaces) = node.implements_interfaces() {
            for implements in implements_interfaces.named_types() {
                let name = implements
                    .name()
                    .ok_or(FromError::MissingNode)?
                    .text()
                    .to_string();
                encoder_node.interface(name);
            }
        }

        if let Some(field_definitions) = node.fields_definition() {
            for field_definition in field_definitions.field_definitions() {
                encoder_node.field(field_definition.try_into()?);
            }
        }

        encoder_node.extend();

        Ok(encoder_node)
    }
}

impl TryFrom<ast::UnionTypeExtension> for crate::UnionDefinition {
    type Error = FromError;

    /// Create an apollo-encoder node from an apollo-parser one.
    ///
    /// # Errors
    /// This returns an error if the apollo-parser tree is not valid. The error
    /// doesn't have much context due to TryFrom API constraints: validate the parse tree before
    /// using TryFrom if granular errors are important to you.
    fn try_from(node: ast::UnionTypeExtension) -> Result<Self, Self::Error> {
        let name = node
            .name()
            .ok_or(FromError::MissingNode)?
            .text()
            .to_string();
        let mut encoder_node = Self::new(name);

        if let Some(directives) = node.directives() {
            for directive in directives.directives() {
                encoder_node.directive(directive.try_into()?);
            }
        }

        if let Some(members) = node.union_member_types() {
            for member in members.named_types() {
                encoder_node.member(
                    member
                        .name()
                        .ok_or(FromError::MissingNode)?
                        .text()
                        .to_string(),
                );
            }
        }

        encoder_node.extend();

        Ok(encoder_node)
    }
}

impl TryFrom<ast::EnumTypeExtension> for crate::EnumDefinition {
    type Error = FromError;

    /// Create an apollo-encoder node from an apollo-parser one.
    ///
    /// # Errors
    /// This returns an error if the apollo-parser tree is not valid. The error
    /// doesn't have much context due to TryFrom API constraints: validate the parse tree before
    /// using TryFrom if granular errors are important to you.
    fn try_from(node: ast::EnumTypeExtension) -> Result<Self, Self::Error> {
        let name = node
            .name()
            .ok_or(FromError::MissingNode)?
            .text()
            .to_string();
        let mut encoder_node = Self::new(name);

        if let Some(directives) = node.directives() {
            for directive in directives.directives() {
                encoder_node.directive(directive.try_into()?);
            }
        }

        if let Some(values) = node.enum_values_definition() {
            for value in values.enum_value_definitions() {
                encoder_node.value(value.try_into()?);
            }
        }

        encoder_node.extend();

        Ok(encoder_node)
    }
}

impl TryFrom<ast::InputObjectTypeExtension> for crate::InputObjectDefinition {
    type Error = FromError;

    /// Create an apollo-encoder node from an apollo-parser one.
    ///
    /// # Errors
    /// This returns an error if the apollo-parser tree is not valid. The error
    /// doesn't have much context due to TryFrom API constraints: validate the parse tree before
    /// using TryFrom if granular errors are important to you.
    fn try_from(node: ast::InputObjectTypeExtension) -> Result<Self, Self::Error> {
        let name = node
            .name()
            .ok_or(FromError::MissingNode)?
            .text()
            .to_string();
        let mut encoder_node = Self::new(name);

        if let Some(directives) = node.directives() {
            for directive in directives.directives() {
                encoder_node.directive(directive.try_into()?);
            }
        }

        if let Some(field_definitions) = node.input_fields_definition() {
            for field_definition in field_definitions.input_value_definitions() {
                encoder_node.field(field_definition.try_into()?);
            }
        }

        encoder_node.extend();

        Ok(encoder_node)
    }
}

impl TryFrom<ast::Document> for crate::Document {
    type Error = FromError;

    /// Create an apollo-encoder node from an apollo-parser one.
    ///
    /// # Errors
    /// This returns an error if the apollo-parser tree is not valid. The error
    /// doesn't have much context due to TryFrom API constraints: validate the parse tree before
    /// using TryFrom if granular errors are important to you.
    fn try_from(node: ast::Document) -> Result<Self, Self::Error> {
        let mut encoder_node = Self::new();

        for definition in node.definitions() {
            match definition {
                ast::Definition::OperationDefinition(def) => {
                    encoder_node.operation(def.try_into()?)
                }
                ast::Definition::FragmentDefinition(def) => encoder_node.fragment(def.try_into()?),
                ast::Definition::DirectiveDefinition(def) => {
                    encoder_node.directive(def.try_into()?)
                }
                ast::Definition::SchemaDefinition(def) => encoder_node.schema(def.try_into()?),
                ast::Definition::ScalarTypeDefinition(def) => encoder_node.scalar(def.try_into()?),
                ast::Definition::ObjectTypeDefinition(def) => encoder_node.object(def.try_into()?),
                ast::Definition::InterfaceTypeDefinition(def) => {
                    encoder_node.interface(def.try_into()?)
                }
                ast::Definition::UnionTypeDefinition(def) => encoder_node.union(def.try_into()?),
                ast::Definition::EnumTypeDefinition(def) => encoder_node.enum_(def.try_into()?),
                ast::Definition::InputObjectTypeDefinition(def) => {
                    encoder_node.input_object(def.try_into()?)
                }
                ast::Definition::SchemaExtension(ext) => encoder_node.schema(ext.try_into()?),
                ast::Definition::ScalarTypeExtension(ext) => encoder_node.scalar(ext.try_into()?),
                ast::Definition::ObjectTypeExtension(ext) => encoder_node.object(ext.try_into()?),
                ast::Definition::InterfaceTypeExtension(ext) => {
                    encoder_node.interface(ext.try_into()?)
                }
                ast::Definition::UnionTypeExtension(ext) => encoder_node.union(ext.try_into()?),
                ast::Definition::EnumTypeExtension(ext) => encoder_node.enum_(ext.try_into()?),
                ast::Definition::InputObjectTypeExtension(ext) => {
                    encoder_node.input_object(ext.try_into()?)
                }
            }
        }

        Ok(encoder_node)
    }
}

#[cfg(test)]
mod tests {
    use crate::Document;
    use apollo_parser::Parser;

    #[test]
    fn operation_definition() {
        let parser = Parser::new(
            r#"
query HeroForEpisode($ep: Episode!) {
  hero(episode: $ep) {
    name
    ... on Droid {
      primaryFunction
    }
    ... on Human {
      height
    }
  }
}
"#,
        );
        let ast = parser.parse();
        let doc = ast.document();

        let encoder = Document::try_from(doc).unwrap();
        assert_eq!(
            encoder.to_string(),
            r#"
query HeroForEpisode($ep: Episode!) {
  hero(episode: $ep) {
    name
    ... on Droid {
      primaryFunction
    }
    ... on Human {
      height
    }
  }
}
"#
            .trim_start()
        );
    }

    #[test]
    fn fragment_definition() {
        let parser = Parser::new(
            r#"
fragment FragmentDefinition on VeryRealType {
  id
  title
  text
}
"#,
        );
        let ast = parser.parse();
        let doc = ast.document();

        let encoder = Document::try_from(doc).unwrap();
        assert_eq!(
            encoder.to_string(),
            r#"
fragment FragmentDefinition on VeryRealType {
  id
  title
  text
}
"#
            .trim_start()
        );
    }

    #[test]
    fn directive_definition() {
        let parser = Parser::new(
            r#"
directive @withDeprecatedArgs(
  deprecatedArg: String @deprecated(reason: "Use `newArg`")
  newArg: String
) on FIELD
"#,
        );
        let ast = parser.parse();
        let doc = ast.document();

        let encoder = Document::try_from(doc).unwrap();
        assert_eq!(encoder.to_string(), r#"
directive @withDeprecatedArgs(deprecatedArg: String @deprecated(reason: "Use `newArg`"), newArg: String) on FIELD
"#.trim_start());
    }

    #[test]
    fn schema_definition() {
        let parser = Parser::new(
            r#"
schema {
  query: Query
  subscription: Subscription
}
extend schema {
  mutation: Mutation
}
"#,
        );
        let ast = parser.parse();
        let doc = ast.document();

        let encoder = Document::try_from(doc).unwrap();
        assert_eq!(
            encoder.to_string(),
            r#"
schema {
  query: Query
  subscription: Subscription
}
extend schema {
  mutation: Mutation
}
"#
            .trim_start()
        );
    }

    #[test]
    fn scalar_definition() {
        let parser = Parser::new(
            r#"
scalar Date
extend scalar Date @directive
"#,
        );
        let ast = parser.parse();
        let doc = ast.document();

        let encoder = Document::try_from(doc).unwrap();
        assert_eq!(
            encoder.to_string(),
            r#"
scalar Date
extend scalar Date @directive
"#
            .trim_start()
        );
    }

    #[test]
    fn object_type_definition() {
        let parser = Parser::new(
            r#"
type User implements X & Y
  @join__owner(graph: USERS)
  @join__type(graph: USERS, key: "email")
{
  email: String! @join__field(graph: USERS)
  id: String! @join__field(graph: USERS)
  name: String @join__field(graph: USERS)
  userProduct: UserProduct @join__field(graph: USERS)
}
"#,
        );
        let ast = parser.parse();
        let doc = ast.document();

        let encoder = Document::try_from(doc).unwrap();
        assert_eq!(
            encoder.to_string(),
            r#"
type User implements X & Y @join__owner(graph: USERS) @join__type(graph: USERS, key: "email") {
  email: String! @join__field(graph: USERS)
  id: String! @join__field(graph: USERS)
  name: String @join__field(graph: USERS)
  userProduct: UserProduct @join__field(graph: USERS)
}
"#
            .trim_start()
        );
    }

    #[test]
    fn interface_definition() {
        let parser = Parser::new(
            r#"
interface X {
  email: String!
}
interface Y {
  id: ID!
}
interface Z implements X & Y @inaccessible {}
"#,
        );
        let ast = parser.parse();
        let doc = ast.document();

        let encoder = Document::try_from(doc).unwrap();
        assert_eq!(
            encoder.to_string(),
            r#"
interface X {
  email: String!
}
interface Y {
  id: ID!
}
interface Z implements X& Y @inaccessible {
}
"#
            .trim_start()
        );
    }

    #[test]
    fn union_definition() {
        let parser = Parser::new(
            r#"
union UnionType = X | Y | Z
"#,
        );
        let ast = parser.parse();
        let doc = ast.document();

        let encoder = Document::try_from(doc).unwrap();
        assert_eq!(
            encoder.to_string(),
            r#"
union UnionType = X | Y | Z
"#
            .trim_start()
        );
    }

    #[test]
    fn enum_definition() {
        let parser = Parser::new(
            r#"
"Documentation for an enum"
enum EnumType {
  X
  "This is Y"
  Y
  Z @test()
}
"#,
        );
        let ast = parser.parse();
        let doc = ast.document();

        let encoder = Document::try_from(doc).unwrap();
        assert_eq!(
            encoder.to_string(),
            r#"
"Documentation for an enum"
enum EnumType {
  X
  "This is Y"
  Y
  Z @test
}
"#
            .trim_start()
        );
    }
}
