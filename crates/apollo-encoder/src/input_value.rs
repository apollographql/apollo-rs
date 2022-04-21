use std::fmt;

use crate::{Directive, StringValue, Type_};

/// The InputValueDefinition type represents field and directive arguments.
///
/// *InputValueDefinition*:
///     Description? Name **:** Type DefaultValue? Directives?
///
/// Detailed documentation can be found in [GraphQL spec](https://spec.graphql.org/October2021/#sec-The-__InputValue-Type).
///
/// ### Example
/// ```rust
/// use apollo_encoder::{Type_, InputValueDefinition};
///
/// let ty_1 = Type_::NamedType {
///     name: "SpaceProgram".to_string(),
/// };
///
/// let ty_2 = Type_::List { ty: Box::new(ty_1) };
/// let mut value = InputValueDefinition::new("cat".to_string(), ty_2);
/// value.description("Very good cats".to_string());
///
/// assert_eq!(
///     value.to_string(),
///     r#""Very good cats" cat: [SpaceProgram]"#
/// );
/// ```
#[derive(Debug, PartialEq, Clone)]
pub struct InputValueDefinition {
    // Name must return a String.
    name: String,
    // Description may return a String.
    pub(crate) description: Option<StringValue>,
    // Type must return a __Type that represents the type this input value expects.
    type_: Type_,
    // Default may return a String encoding (using the GraphQL language) of
    // the default value used by this input value in the condition a value is
    // not provided at runtime. If this input value has no default value,
    // returns null.
    default: Option<String>,
    /// Contains all directives for this input value definition
    directives: Vec<Directive>,
}

impl InputValueDefinition {
    /// Create a new instance of InputValueDefinition.
    pub fn new(name: String, type_: Type_) -> Self {
        Self {
            description: None,
            name,
            type_,
            default: None,
            directives: Vec::new(),
        }
    }

    /// Set the InputValueDefinition's description.
    pub fn description(&mut self, description: String) {
        self.description = Some(StringValue::Input {
            source: description,
        });
    }

    /// Set the InputValueDefinition's default value.
    pub fn default(&mut self, default: Option<String>) {
        self.default = default;
    }

    /// Add a directive to InputValueDefinition.
    pub fn directive(&mut self, directive: Directive) {
        self.directives.push(directive)
    }
}

impl fmt::Display for InputValueDefinition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.description {
            Some(description) => {
                writeln!(f, "\n    {}", description)?;

                write!(f, "    {}: {}", self.name, self.type_)?;
            }
            None => {
                write!(f, "{}: {}", self.name, self.type_)?;
            }
        }

        if let Some(default) = &self.default {
            write!(f, " = {}", default)?;
        }

        for directive in &self.directives {
            write!(f, " {}", directive)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{Argument, Value};

    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn it_encodes_simple_values() {
        let ty_1 = Type_::NamedType {
            name: "SpaceProgram".to_string(),
        };

        let ty_2 = Type_::List { ty: Box::new(ty_1) };
        let ty_3 = Type_::NonNull { ty: Box::new(ty_2) };
        let value = InputValueDefinition::new("spaceCat".to_string(), ty_3);

        assert_eq!(value.to_string(), r#"spaceCat: [SpaceProgram]!"#);
    }

    #[test]
    fn it_encodes_input_values_with_default() {
        let ty_1 = Type_::NamedType {
            name: "Breed".to_string(),
        };

        let ty_2 = Type_::NonNull { ty: Box::new(ty_1) };
        let mut value = InputValueDefinition::new("spaceCat".to_string(), ty_2);
        value.default(Some("\"Norwegian Forest\"".to_string()));

        assert_eq!(
            value.to_string(),
            r#"spaceCat: Breed! = "Norwegian Forest""#
        );
    }

    #[test]
    fn it_encodes_value_with_directive() {
        let ty_1 = Type_::NamedType {
            name: "SpaceProgram".to_string(),
        };

        let ty_2 = Type_::List { ty: Box::new(ty_1) };
        let mut value = InputValueDefinition::new("cat".to_string(), ty_2);
        let mut directive = Directive::new(String::from("testDirective"));
        directive.arg(Argument::new(String::from("first"), Value::Int(1)));
        value.description("Very good cats".to_string());
        value.directive(directive);

        assert_eq!(
            value.to_string(),
            r#"
    "Very good cats"
    cat: [SpaceProgram] @testDirective(first: 1)"#
        );
    }

    #[test]
    fn it_encodes_valueuments_with_description() {
        let ty_1 = Type_::NamedType {
            name: "SpaceProgram".to_string(),
        };

        let ty_2 = Type_::NonNull { ty: Box::new(ty_1) };
        let ty_3 = Type_::List { ty: Box::new(ty_2) };
        let ty_4 = Type_::NonNull { ty: Box::new(ty_3) };
        let mut value = InputValueDefinition::new("spaceCat".to_string(), ty_4);
        value.description("Very good space cats".to_string());

        assert_eq!(
            value.to_string(),
            r#"
    "Very good space cats"
    spaceCat: [SpaceProgram!]!"#
        );
    }
}
