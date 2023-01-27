use std::fmt;

use crate::Argument;

/// The `Directive` type represents a Directive, it provides a way to describe alternate runtime execution and type validation behavior in a GraphQL document.
///
/// *Directive*:
///     @ Name Arguments?
///
/// Detailed documentation can be found in [GraphQL spec](https://spec.graphql.org/October2021/#sec-Language.Directives).
///
/// ### Example
/// ```rust
/// use apollo_encoder::{Argument, Directive, Value};
///
/// let mut directive = Directive::new(String::from("myDirective"));
/// directive.arg(Argument::new(String::from("first"), Value::Int(5)));
///
/// assert_eq!(directive.to_string(), "@myDirective(first: 5)");
/// ```
#[derive(Debug, PartialEq, Clone)]
pub struct Directive {
    name: String,
    args: Vec<Argument>,
}

impl Directive {
    /// Create an instance of Directive
    pub fn new(name: String) -> Self {
        Self {
            name,
            args: Vec::new(),
        }
    }

    /// Add an argument to the directive
    pub fn arg(&mut self, arg: Argument) {
        self.args.push(arg);
    }
}

impl fmt::Display for Directive {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "@{}", self.name)?;

        if !self.args.is_empty() {
            for (i, arg) in self.args.iter().enumerate() {
                match i {
                    0 => write!(f, "({arg}")?,
                    _ => write!(f, ", {arg}")?,
                }
            }
            write!(f, ")")?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::Value;

    use super::*;

    #[test]
    fn it_encodes_directive() {
        let mut directive = Directive::new(String::from("myDirective"));
        directive.arg(Argument::new(String::from("first"), Value::Int(5)));

        assert_eq!(directive.to_string(), "@myDirective(first: 5)");
    }
}
