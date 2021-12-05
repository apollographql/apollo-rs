use std::fmt;

use crate::{StringValue, Type_};

/// Input Field in a given Input Object.
/// A GraphQL Input Object defines a set of input fields; the input fields are
/// either scalars, enums, or other input objects. Input fields are similar to
/// Fields, but can have a default value.
#[derive(Debug, PartialEq, Clone)]
pub struct InputField {
    // Name must return a String.
    name: String,
    // Description may return a String.
    description: StringValue,
    // Type must return a __Type that represents the type of value returned by this field.
    type_: Type_,
    // Default value for this input field.
    default_value: Option<String>,
}

/// ### Example
/// ```rust
/// use apollo_encoder::{Type_, InputFieldBuilder};
///
/// let ty = Type_::named("CatBreed");
///
/// let field = InputFieldBuilder::new("cat", ty)
///     .default("\"Norwegian Forest\"")
///     .build();
///
/// assert_eq!(field.to_string(), r#"  cat: CatBreed = "Norwegian Forest""#);
/// ```
#[derive(Debug, Clone)]
pub struct InputFieldBuilder {
    // Name must return a String.
    name: String,
    // Description may return a String.
    description: Option<String>,
    // Type must return a __Type that represents the type of value returned by this field.
    type_: Type_,
    // Default value for this input field.
    default_value: Option<String>,
}

impl InputFieldBuilder {
    /// Create a new instance of InputFieldBuilder.
    pub fn new(name: &str, type_: Type_) -> Self {
        Self {
            name: name.to_string(),
            description: None,
            type_,
            default_value: None,
        }
    }

    /// Set the InputField's description.
    pub fn description(mut self, description: &str) -> Self {
        self.description = Some(description.to_string());
        self
    }

    /// Set the InputField's default value.
    pub fn default(mut self, default: &str) -> Self {
        self.default_value = Some(default.to_string());
        self
    }

    /// Create a new instance of InputField.
    pub fn build(self) -> InputField {
        InputField {
            name: self.name,
            description: StringValue::Field {
                source: self.description,
            },
            type_: self.type_,
            default_value: self.default_value,
        }
    }
}

impl fmt::Display for InputField {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.description)?;

        write!(f, "  {}: {}", self.name, self.type_)?;
        if let Some(default) = &self.default_value {
            write!(f, " = {}", default)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{InputFieldBuilder, Type_};
    use pretty_assertions::assert_eq;

    #[test]
    fn it_encodes_fields_with_defaults() {
        let ty = Type_::named("CatBreed");

        let field = InputFieldBuilder::new("cat", ty)
            .default("\"Norwegian Forest\"")
            .build();

        assert_eq!(field.to_string(), r#"  cat: CatBreed = "Norwegian Forest""#);
    }
}
