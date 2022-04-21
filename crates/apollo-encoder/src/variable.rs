use std::fmt;

use crate::{Directive, Type_, Value};

/// The VariableDefinition type represents a variable definition
///
/// *VariableDefinition*:
///     VariableName : Type DefaultValue? Directives?
///
/// Detailed documentation can be found in [GraphQL spec](https://spec.graphql.org/October2021/#sec-Language.Variables).
///
/// ### Example
/// ```rust
/// use apollo_encoder::{Type_, Value, VariableDefinition};
///
/// let mut variable = VariableDefinition::new(
///     String::from("my_var"),
///     Type_::NamedType {
///         name: String::from("MyType"),
///     },
/// );
/// variable.default_value(Some(Value::Object(vec![
///     (String::from("first"), Value::Int(25)),
///     (String::from("second"), Value::String(String::from("test"))),
/// ])));
///
/// assert_eq!(
///     variable.to_string(),
///     String::from(r#"$my_var: MyType = { first: 25, second: "test" }"#)
/// );
/// ```
#[derive(Debug)]
pub struct VariableDefinition {
    variable: String,
    ty: Type_,
    default_value: Option<Value>,
    directives: Vec<Directive>,
}

impl VariableDefinition {
    /// Create an instance of VariableDefinition
    pub fn new(variable: String, ty: Type_) -> Self {
        Self {
            variable,
            ty,
            default_value: Option::default(),
            directives: Vec::new(),
        }
    }

    /// Set a default value to the variable
    pub fn default_value(&mut self, default_value: Value) {
        self.default_value = Some(default_value);
    }

    /// Add a directive
    pub fn directive(&mut self, directive: Directive) {
        self.directives.push(directive);
    }
}

impl fmt::Display for VariableDefinition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "${}: {}", self.variable, self.ty)?;

        if let Some(default_value) = &self.default_value {
            write!(f, " = {}", default_value)?;
        }

        for directive in &self.directives {
            write!(f, " {}", directive)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_encodes_variable_definition() {
        let mut variable = VariableDefinition::new(
            String::from("my_var"),
            Type_::NamedType {
                name: String::from("MyType"),
            },
        );
        variable.default_value(Value::Object(vec![
            (String::from("first"), Value::Int(25)),
            (String::from("second"), Value::String(String::from("test"))),
        ]));

        assert_eq!(
            variable.to_string(),
            String::from(r#"$my_var: MyType = { first: 25, second: "test" }"#)
        );
    }
}
