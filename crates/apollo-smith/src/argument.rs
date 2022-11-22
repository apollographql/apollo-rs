use arbitrary::Result;

use crate::{
    input_value::{InputValue, InputValueDef},
    name::Name,
    DocumentBuilder,
};

/// The `__ArgumentsDef` type represents an arguments definition
///
/// *ArgumentsDefinition*:
///     ( InputValueDefinition* )
///
/// Detailed documentation can be found in [GraphQL spec](https://spec.graphql.org/October2021/#ArgumentsDefinition).
///
#[derive(Debug, Clone, PartialEq)]
pub struct ArgumentsDef {
    pub(crate) input_value_definitions: Vec<InputValueDef>,
}

impl From<ArgumentsDef> for apollo_encoder::ArgumentsDefinition {
    fn from(args_def: ArgumentsDef) -> Self {
        apollo_encoder::ArgumentsDefinition::with_values(
            args_def
                .input_value_definitions
                .into_iter()
                .map(Into::into)
                .collect(),
        )
    }
}

#[cfg(feature = "parser-impl")]
impl TryFrom<apollo_parser::ast::ArgumentsDefinition> for ArgumentsDef {
    type Error = crate::FromError;

    fn try_from(args_def: apollo_parser::ast::ArgumentsDefinition) -> Result<Self, Self::Error> {
        Ok(Self {
            input_value_definitions: args_def
                .input_value_definitions()
                .map(InputValueDef::try_from)
                .collect::<Result<_, _>>()?,
        })
    }
}

/// The `__Argument` type represents an argument
///
/// *Argument*:
///     Name: Value
///
/// Detailed documentation can be found in [GraphQL spec](https://spec.graphql.org/October2021/#sec-Language.Arguments).
#[derive(Debug, Clone, PartialEq)]
pub struct Argument {
    pub(crate) name: Name,
    pub(crate) value: InputValue,
}

impl From<Argument> for apollo_encoder::Argument {
    fn from(arg: Argument) -> Self {
        Self::new(arg.name.into(), arg.value.into())
    }
}

#[cfg(feature = "parser-impl")]
impl TryFrom<apollo_parser::ast::Argument> for Argument {
    type Error = crate::FromError;

    fn try_from(argument: apollo_parser::ast::Argument) -> Result<Self, Self::Error> {
        Ok(Self {
            name: argument.name().unwrap().into(),
            value: argument.value().unwrap().try_into()?,
        })
    }
}

impl<'a> DocumentBuilder<'a> {
    /// Create an arbitrary vector of `Argument`
    pub fn arguments(&mut self) -> Result<Vec<Argument>> {
        let num_arguments = self.u.int_in_range(0..=4)?;
        let arguments = (0..num_arguments)
            .map(|_| self.argument())
            .collect::<Result<Vec<_>>>()?;

        Ok(arguments)
    }

    /// Create an arbitrary vector of `Argument` given ArgumentsDef
    pub fn arguments_with_def(&mut self, args_def: &ArgumentsDef) -> Result<Vec<Argument>> {
        let arguments = args_def
            .input_value_definitions
            .iter()
            .map(|input_val_def| self.argument_with_def(input_val_def))
            .collect::<Result<Vec<_>>>()?;

        Ok(arguments)
    }

    /// Create an arbitrary `Argument`
    pub fn argument(&mut self) -> Result<Argument> {
        let name = self.name()?;
        let value = self.input_value()?;

        Ok(Argument { name, value })
    }

    /// Create an arbitrary `Argument`
    pub fn argument_with_def(&mut self, input_val_def: &InputValueDef) -> Result<Argument> {
        let name = input_val_def.name.clone();
        let value = self.input_value_for_type(&input_val_def.ty)?;

        Ok(Argument { name, value })
    }

    /// Create an arbitrary `ArgumentsDef`
    pub fn arguments_definition(&mut self) -> Result<ArgumentsDef> {
        Ok(ArgumentsDef {
            input_value_definitions: self.input_values_def()?,
        })
    }
}
