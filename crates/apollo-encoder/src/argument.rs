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
/// let arguments_def = ArgumentsDefinition::new(input_value_defs);
///
/// assert_eq!(arguments_def.to_string(), r#"(first: Int, second: [Int])"#);
/// ```
#[derive(Debug)]
pub struct ArgumentsDefinition {
    input_value_definitions: Vec<InputValueDefinition>,
}

impl ArgumentsDefinition {
    /// Create a new instance of Argument definition.
    pub fn new(input_value_definitions: Vec<InputValueDefinition>) -> Self {
        Self {
            input_value_definitions,
        }
    }
}

impl fmt::Display for ArgumentsDefinition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "(")?;
        if self.input_value_definitions.len() == 1 {
            write!(f, "{}, ", self.input_value_definitions[0])?;
        } else {
            for (i, input_val_def) in self.input_value_definitions.iter().enumerate() {
                if i != self.input_value_definitions.len() - 1 {
                    write!(f, "{}, ", input_val_def)?;
                } else {
                    write!(f, "{}", input_val_def)?;
                }
            }
        }
        write!(f, ")")
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
        let arguments_def = ArgumentsDefinition::new(input_value_defs);

        assert_eq!(arguments_def.to_string(), r#"(first: Int, second: [Int])"#);
    }
}
