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
        apollo_encoder::ArgumentsDefinition::new(
            args_def
                .input_value_definitions
                .into_iter()
                .map(Into::into)
                .collect(),
        )
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

impl<'a> DocumentBuilder<'a> {
    /// Create an arbitrary vector of `Argument`
    pub fn arguments(&mut self) -> Result<Vec<Argument>> {
        let num_arguments = self.u.int_in_range(0..=4)?;
        let arguments = (0..num_arguments)
            .map(|_| self.argument())
            .collect::<Result<Vec<_>>>()?;

        Ok(arguments)
    }

    /// Create an arbitrary `Argument`
    pub fn argument(&mut self) -> Result<Argument> {
        let name = self.name()?;
        let value = self.input_value()?;

        Ok(Argument { name, value })
    }

    /// Create an arbitrary `ArgumentsDef`
    pub fn arguments_definition(&mut self) -> Result<ArgumentsDef> {
        Ok(ArgumentsDef {
            input_value_definitions: self.input_values_def()?,
        })
    }
}
