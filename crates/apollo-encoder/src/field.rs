use std::fmt;

use crate::{InputValue, StringValue, Type_};
/// The __Field type represents each field in an Object or Interface type.
///
/// *FieldDefinition*:
///     Description? Name ArgumentsDefinition? **:** TypeDirectives?
///
/// Detailed documentation can be found in [GraphQL spec](https://spec.graphql.org/October2021/#sec-The-__Field-Type).
///
/// ### Example
/// ```rust
/// use apollo_encoder::{Type_, Field, InputValue};
///
/// let ty_1 = Type_::named_type("CatBreed");
///
/// let mut field = Field::new("cat", ty_1);
///
/// let value_1 = Type_::named_type("CatBreed");
///
/// let arg = InputValue::new("breed", value_1);
///
/// field.arg(arg);
///
/// assert_eq!(
///     field.to_string(),
///     r#"  cat(breed: CatBreed): CatBreed"#
/// );
/// ```
#[derive(Debug, PartialEq, Clone)]
pub struct Field {
    // Name must return a String.
    name: String,
    // Description may return a String.
    description: StringValue,
    // Args returns a List of __InputValue representing the arguments this field accepts.
    args: Vec<InputValue>,
    // Type must return a __Type that represents the type of value returned by this field.
    type_: Type_,
    // Deprecated returns true if this field should no longer be used, otherwise false.
    is_deprecated: bool,
    // Deprecation reason optionally provides a reason why this field is deprecated.
    deprecation_reason: StringValue,
}

#[derive(Debug, Clone)]
pub struct FieldBuilder {
    // Name must return a String.
    name: String,
    // Description may return a String.
    description: Option<String>,
    // Args returns a List of __InputValue representing the arguments this field accepts.
    args: Vec<InputValue>,
    // Type must return a __Type that represents the type of value returned by this field.
    type_: Type_,
    // Deprecation reason optionally provides a reason why this field is deprecated.
    deprecation_reason: Option<String>,
}

impl FieldBuilder {
    /// Create a new instance of FieldBuilder.
    pub fn new(name: &str, type_: Type_) -> Self {
        Self {
            name: name.to_string(),
            description: None,
            args: Vec::new(),
            type_,
            deprecation_reason: None,
        }
    }

    /// Set the Field's description.
    pub fn description(mut self, description: &str) -> Self {
        self.description = Some(description.to_string());
        self
    }

    /// Set the Field's deprecation properties.
    pub fn deprecated(mut self, reason: &str) -> Self {
        self.deprecation_reason = Some(reason.to_string());
        self
    }

    /// Set the Field's arguments.
    pub fn arg(mut self, arg: InputValue) -> Self {
        self.args.push(arg);
        self
    }

    /// Create a new instance of Field.
    pub fn build(self) -> Field {
        Field {
            name: self.name,
            description: StringValue::Field {
                source: self.description,
            },
            args: self.args,
            type_: self.type_,
            is_deprecated: self.deprecation_reason.is_some(),
            deprecation_reason: StringValue::Reason {
                source: self.deprecation_reason,
            },
        }
    }
}

impl fmt::Display for Field {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.description)?;
        write!(f, "  {}", self.name)?;

        if !self.args.is_empty() {
            for (i, arg) in self.args.iter().enumerate() {
                match i {
                    0 => write!(f, "({}", arg)?,
                    _ => write!(f, ", {}", arg)?,
                }
            }
            write!(f, ")")?;
        }

        write!(f, ": {}", self.type_)?;

        if self.is_deprecated {
            write!(f, " @deprecated")?;

            if let StringValue::Reason { source: _ } = &self.deprecation_reason {
                write!(f, "(reason:")?;
                write!(f, "{}", self.deprecation_reason)?;
                write!(f, ")")?
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn it_encodes_simple_fields() {
        let field = {
            let ty = Type_::named_type("SpaceProgram");
            let ty = Type_::list(Box::new(ty));
            let ty = Type_::non_null(Box::new(ty));

            Field::new("spaceCat", ty)
        };

        assert_eq!(field.to_string(), r#"  spaceCat: [SpaceProgram]!"#);
    }

    #[test]
    fn it_encodes_fields_with_deprecation() {
        let field = {
            let ty = Type_::named_type("SpaceProgram");
            let ty = Type_::list(Box::new(ty));

            let mut field = Field::new("cat", ty);
            field.description("Very good cats");
            field.deprecated("Cats are no longer sent to space.");
            field
        };

        assert_eq!(
            field.to_string(),
            r#"  "Very good cats"
  cat: [SpaceProgram] @deprecated(reason: "Cats are no longer sent to space.")"#
        );
    }

    #[test]
    fn it_encodes_fields_with_description() {
        let field = {
            let ty = Type_::named_type("SpaceProgram");
            let ty = Type_::non_null(Box::new(ty));
            let ty = Type_::list(Box::new(ty));
            let ty = Type_::non_null(Box::new(ty));

            let mut field = Field::new("spaceCat", ty);
            field.description("Very good space cats");
            field
        };

        assert_eq!(
            field.to_string(),
            r#"  "Very good space cats"
  spaceCat: [SpaceProgram!]!"#
        );
    }

    #[test]
    fn it_encodes_fields_with_valueuments() {
        let field = {
            let ty = {
                let ty = Type_::named_type("SpaceProgram");
                let ty = Type_::non_null(Box::new(ty));
                let ty = Type_::list(Box::new(ty));
                let ty = Type_::non_null(Box::new(ty));
                ty
            };

            let mut field = Field::new("spaceCat", ty);
            field.description("Very good space cats");

            let arg = {
                let value = Type_::named_type("SpaceProgram");
                let value = Type_::list(Box::new(value));

                let mut arg = InputValue::new("cat", value);
                arg.deprecated("Cats are no longer sent to space.");
                arg
            };
            field.arg(arg);
            field
        };

        assert_eq!(
            field.to_string(),
            r#"  "Very good space cats"
  spaceCat(cat: [SpaceProgram] @deprecated(reason: "Cats are no longer sent to space.")): [SpaceProgram!]!"#
        );
    }
}
