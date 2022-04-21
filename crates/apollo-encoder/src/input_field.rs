use std::fmt;

use crate::{Directive, StringValue, Type_};

#[derive(Debug, PartialEq, Clone)]
/// Input Field in a given Input Object.
/// A GraphQL Input Object defines a set of input fields; the input fields are
/// either scalars, enums, or other input objects. Input fields are similar to
/// Fields, but can have a default value.
///
/// ### Example
/// ```rust
/// use apollo_encoder::{Type_, InputField};
///
/// let ty_1 = Type_::NamedType {
///     name: "CatBreed".to_string(),
/// };
///
/// let mut field = InputField::new("cat".to_string(), ty_1);
/// field.default(Some("\"Norwegian Forest\"".to_string()));
///
/// assert_eq!(field.to_string(), r#"  cat: CatBreed = "Norwegian Forest""#);
/// ```
pub struct InputField {
    // Name must return a String.
    name: String,
    // Description may return a String.
    description: Option<StringValue>,
    // Type must return a __Type that represents the type of value returned by this field.
    type_: Type_,
    // Default value for this input field.
    default_value: Option<String>,
    /// Contains all directives for this input value definition
    directives: Vec<Directive>,
}

impl InputField {
    /// Create a new instance of InputField.
    pub fn new(name: String, type_: Type_) -> Self {
        Self {
            description: None,
            name,
            type_,
            default_value: None,
            directives: Vec::new(),
        }
    }

    /// Set the InputField's description.
    pub fn description(&mut self, description: String) {
        self.description = Some(StringValue::Field {
            source: description,
        });
    }

    /// Set the InputField's default value.
    pub fn default(&mut self, default: Option<String>) {
        self.default_value = default;
    }

    /// Add a directive.
    pub fn directive(&mut self, directive: Directive) {
        self.directives.push(directive)
    }
}

impl fmt::Display for InputField {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(description) = &self.description {
            write!(f, "{}", description)?;
        }

        write!(f, "  {}: {}", self.name, self.type_)?;
        if let Some(default) = &self.default_value {
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
    fn it_encodes_fields_with_defaults() {
        let ty_1 = Type_::NamedType {
            name: "CatBreed".to_string(),
        };

        let mut field = InputField::new("cat".to_string(), ty_1);
        field.default(Some("\"Norwegian Forest\"".to_string()));

        assert_eq!(field.to_string(), r#"  cat: CatBreed = "Norwegian Forest""#);
    }
    #[test]
    fn it_encodes_fields_with_directives() {
        let ty_1 = Type_::NamedType {
            name: "CatBreed".to_string(),
        };
        let mut directive = Directive::new(String::from("testDirective"));
        directive.arg(Argument::new(String::from("first"), Value::Int(1)));

        let mut field = InputField::new("cat".to_string(), ty_1);
        field.default(Some("\"Norwegian Forest\"".to_string()));
        field.directive(directive);

        assert_eq!(
            field.to_string(),
            r#"  cat: CatBreed = "Norwegian Forest" @testDirective(first: 1)"#
        );
    }
}
