use std::fmt;

use crate::{InputValueDefinition, Value};

/// The `ArgumentsDefinition` type represents an arguments definition
///
/// *ArgumentsDefinition*:
///     ( InputValueDefinition* )
///
/// Detailed documentation can be found in [GraphQL spec](https://spec.graphql.org/October2021/#ArgumentsDefinition).
///
/// ### Example
/// ```rust
/// use apollo_encoder::{ArgumentsDefinition, InputValueDefinition, Type_};
/// use indoc::indoc;
///
/// let input_value_defs = vec![
///     InputValueDefinition::new(
///         String::from("first"),
///         Type_::NamedType {
///             name: String::from("Int"),
///         },
///     ),
///     InputValueDefinition::new(
///         String::from("second"),
///         Type_::List {
///             ty: Box::new(Type_::NamedType {
///                 name: String::from("Int"),
///             }),
///         },
///     ),
/// ];
/// let arguments_def = ArgumentsDefinition::with_values(input_value_defs);
///
/// assert_eq!(arguments_def.to_string(), r#"(first: Int, second: [Int])"#);
/// ```
#[derive(Debug, Default, PartialEq, Clone)]
pub struct ArgumentsDefinition {
    pub(crate) input_values: Vec<InputValueDefinition>,
}

impl ArgumentsDefinition {
    /// Create a new instance of Argument definition.
    pub fn new() -> Self {
        Self {
            input_values: Vec::new(),
        }
    }

    /// Create a new instance of ArgumentsDefinition given Input Value Definitions.
    pub fn with_values(input_values: Vec<InputValueDefinition>) -> Self {
        Self { input_values }
    }

    /// Add an InputValueDefinition to Arguments Definition
    pub fn input_value(&mut self, input_value: InputValueDefinition) {
        self.input_values.push(input_value)
    }
}

impl fmt::Display for ArgumentsDefinition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "(")?;
        for (i, input_val_def) in self.input_values.iter().enumerate() {
            // this branch multilines input value definitions, like this:
            //   two(
            //     """This is a description of the \`argument\` argument."""
            //     argument: InputType!
            //   ): Type
            if input_val_def.description.is_some() {
                if i != self.input_values.len() - 1 {
                    write!(f, "{input_val_def},")?;
                } else {
                    writeln!(f, "{input_val_def}")?;
                }
            // with no descriptions we single line input value definitions:
            //   two(argument: InputType!): Type
            } else if i != self.input_values.len() - 1 {
                write!(f, "{input_val_def}, ")?;
            } else {
                write!(f, "{input_val_def}")?;
            }
        }

        if self
            .input_values
            .iter()
            .any(|input| input.description.is_some())
        {
            write!(f, "  )")
        } else {
            write!(f, ")")
        }
    }
}

/// The `Argument` type represents an argument
///
/// *Argument*:
///     Name: Value
///
/// Detailed documentation can be found in [GraphQL spec](https://spec.graphql.org/October2021/#sec-Language.Arguments).
///
/// ### Example
/// ```rust
/// use apollo_encoder::{Argument, Value};
///
/// let argument = Argument::new(String::from("argName"), Value::String("value".to_string()));
/// assert_eq!(argument.to_string(), r#"argName: "value""#);
/// ```
#[derive(Debug, PartialEq, Clone)]
pub struct Argument {
    name: String,
    value: Value,
}

impl Argument {
    /// Create a new instance of Argument.
    pub fn new(name: String, value: Value) -> Self {
        Self { name, value }
    }
}

impl fmt::Display for Argument {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.name, self.value)
    }
}

#[cfg(test)]
mod tests {
    use crate::Type_;

    use super::*;

    #[test]
    fn it_encodes_argument() {
        let argument = Argument::new(String::from("argName"), Value::String("value".to_string()));

        assert_eq!(argument.to_string(), r#"argName: "value""#);
    }

    #[test]
    fn it_encodes_arguments_definitions() {
        let input_value_defs = vec![
            InputValueDefinition::new(
                String::from("first"),
                Type_::NamedType {
                    name: String::from("Int"),
                },
            ),
            InputValueDefinition::new(
                String::from("second"),
                Type_::List {
                    ty: Box::new(Type_::NamedType {
                        name: String::from("Int"),
                    }),
                },
            ),
        ];
        let arguments_def = ArgumentsDefinition::with_values(input_value_defs);

        assert_eq!(arguments_def.to_string(), r#"(first: Int, second: [Int])"#);
    }
}
