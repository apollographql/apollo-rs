use std::fmt;

use crate::{StringValue, Type_};

// NOTE(@lrlna): __InputValue is also meant to be used for InputFields on an
// InputObject. We currently do not differentiate between InputFields and
// Fields, so this is not applied directly to InputObjects. Once we are able to
// walk an AST to encode a schema, we will want to make sure this struct is used
// directly: InputObject --> InputField --> InputValue

/// The __InputValue type represents field and directive arguments.
///
/// *InputValueDefinition*:
///     Description? Name **:** Type DefaultValue? Directives?
///
/// Detailed documentation can be found in [GraphQL spec](https://spec.graphql.org/October2021/#sec-The-__InputValue-Type).
#[derive(Debug, PartialEq, Clone)]
pub struct InputValue {
    // Name must return a String.
    name: String,
    // Description may return a String.
    description: StringValue,
    // Type must return a __Type that represents the type this input value expects.
    type_: Type_,
    // Default may return a String encoding (using the GraphQL language) of
    // the default value used by this input value in the condition a value is
    // not provided at runtime. If this input value has no default value,
    // returns null.
    default: Option<String>,
    // Deprecated returns true if this field should no longer be used, otherwise false.
    is_deprecated: bool,
    // Deprecation reason optionally provides a reason why this field is deprecated.
    deprecation_reason: Option<String>,
}

/// ### Example
/// ```rust
/// use apollo_encoder::{Type_, InputValueBuilder};
///
/// let ty = Type_::named_type("SpaceProgram");
/// let ty = Type_::list(Box::new(ty));
/// let value = InputValueBuilder::new("cat", ty)
///     .description("Very good cats")
///     .deprecated("Cats are no longer sent to space.")
///     .build();
///
/// assert_eq!(
///     value.to_string(),
///     r#""Very good cats" cat: [SpaceProgram] @deprecated(reason: "Cats are no longer sent to space.")"#
/// );
/// ```
#[derive(Debug, Clone)]
pub struct InputValueBuilder {
    // Name must return a String.
    name: String,
    // Description may return a String.
    description: Option<String>,
    // Type must return a __Type that represents the type this input value expects.
    type_: Type_,
    // Default may return a String encoding (using the GraphQL language) of
    // the default value used by this input value in the condition a value is
    // not provided at runtime. If this input value has no default value,
    // returns null.
    default: Option<String>,
    // Deprecation reason optionally provides a reason why this field is deprecated.
    deprecation_reason: Option<String>,
}

impl InputValueBuilder {
    /// Create a new instance of InputValueBuilder.
    pub fn new(name: &str, type_: Type_) -> Self {
        Self {
            description: None,
            name: name.to_string(),
            type_,
            deprecation_reason: None,
            default: None,
        }
    }

    /// Set the InputValue's description.
    pub fn description(mut self, description: &str) -> Self {
        self.description = Some(description.to_string());
        self
    }

    /// Set the InputValue's default value.
    pub fn default(mut self, default: &str) -> Self {
        self.default = Some(default.to_string());
        self
    }

    /// Set the InputValue's deprecation properties.
    pub fn deprecated(mut self, reason: &str) -> Self {
        self.deprecation_reason = Some(reason.to_string());
        self
    }

    /// Create a new instance of InputValue.
    pub fn build(self) -> InputValue {
        InputValue {
            name: self.name,
            description: StringValue::Input {
                source: self.description,
            },
            type_: self.type_,
            is_deprecated: self.deprecation_reason.is_some(),
            deprecation_reason: self.deprecation_reason,
            default: self.default,
        }
    }
}

impl fmt::Display for InputValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.description)?;

        write!(f, "{}: {}", self.name, self.type_)?;

        if let Some(default) = &self.default {
            write!(f, " = {}", default)?;
        }

        if self.is_deprecated {
            write!(f, " @deprecated")?;
            // Just in case deprecated field is ever used without a reason,
            // let's properly unwrap this Option.
            if let Some(reason) = &self.deprecation_reason {
                write!(f, "(reason: \"{}\")", reason)?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{InputValueBuilder, Type_};
    use pretty_assertions::assert_eq;

    #[test]
    fn it_encodes_simple_values() {
        let ty = Type_::named_type("SpaceProgram");
        let ty = Type_::list(Box::new(ty));
        let ty = Type_::non_null(Box::new(ty));

        let value = InputValueBuilder::new("spaceCat", ty).build();

        assert_eq!(value.to_string(), r#"spaceCat: [SpaceProgram]!"#);
    }

    #[test]
    fn it_encodes_input_values_with_default() {
        let ty = Type_::named_type("Breed");
        let ty = Type_::non_null(Box::new(ty));

        let value = InputValueBuilder::new("spaceCat", ty)
            .default("\"Norwegian Forest\"")
            .build();

        assert_eq!(
            value.to_string(),
            r#"spaceCat: Breed! = "Norwegian Forest""#
        );
    }

    #[test]
    fn it_encodes_value_with_deprecation() {
        let ty = Type_::named_type("SpaceProgram");
        let ty = Type_::list(Box::new(ty));

        let value = InputValueBuilder::new("cat", ty)
            .description("Very good cats")
            .deprecated("Cats are no longer sent to space.")
            .build();

        assert_eq!(
            value.to_string(),
            r#""Very good cats" cat: [SpaceProgram] @deprecated(reason: "Cats are no longer sent to space.")"#
        );
    }

    #[test]
    fn it_encodes_valueuments_with_description() {
        let ty = Type_::named_type("SpaceProgram");
        let ty = Type_::non_null(Box::new(ty));
        let ty = Type_::list(Box::new(ty));
        let ty = Type_::non_null(Box::new(ty));

        let value = InputValueBuilder::new("spaceCat", ty)
            .description("Very good space cats")
            .build();

        assert_eq!(
            value.to_string(),
            r#""Very good space cats" spaceCat: [SpaceProgram!]!"#
        );
    }
}
