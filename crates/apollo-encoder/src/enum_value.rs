use std::fmt;

use crate::{Directive, StringValue};

/// The EnumValue type represents one of possible values of an enum.
///
/// *EnumValueDefinition*:
///     Description? EnumValue Directives?
///
/// Detailed documentation can be found in [GraphQL spec](https://spec.graphql.org/October2021/#sec-The-__EnumValue-Type).
///
/// ### Example
/// ```rust
/// use apollo_encoder::{Argument, Directive, EnumValue, Value};
///
/// let mut enum_ty = EnumValue::new("CARDBOARD_BOX".to_string());
/// enum_ty.description("Box nap spot.".to_string());
/// let mut deprecated_directive = Directive::new(String::from("deprecated"));
/// deprecated_directive.arg(Argument::new(
///     String::from("reason"),
///     Value::String(String::from(
///         "Box was recycled.",
///     )),
/// ));
/// enum_ty.directive(deprecated_directive);
///
/// assert_eq!(
///     enum_ty.to_string(),
///     r#"  "Box nap spot."
///   CARDBOARD_BOX @deprecated(reason: "Box was recycled.")"#
/// );
/// ```
#[derive(Debug, PartialEq, Clone)]
pub struct EnumValue {
    // Name must return a String.
    name: String,
    // Description may return a String or null.
    description: Option<StringValue>,
    /// The vector of directives
    directives: Vec<Directive>,
}

impl EnumValue {
    /// Create a new instance of EnumValue.
    pub fn new(name: String) -> Self {
        Self {
            name,
            description: None,
            directives: Vec::new(),
        }
    }

    /// Set the Enum Value's description.
    pub fn description(&mut self, description: String) {
        self.description = Some(StringValue::Field {
            source: description,
        });
    }

    /// Add a directive.
    pub fn directive(&mut self, directive: Directive) {
        self.directives.push(directive)
    }
}

impl fmt::Display for EnumValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(description) = &self.description {
            write!(f, "{description}")?;
        }
        write!(f, "  {}", self.name)?;

        for directive in &self.directives {
            write!(f, " {directive}")?;
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
    fn it_encodes_an_enum_value() {
        let enum_ty = EnumValue::new("CAT_TREE".to_string());
        assert_eq!(enum_ty.to_string(), "  CAT_TREE");
    }

    #[test]
    fn it_encodes_an_enum_value_with_desciption() {
        let mut enum_ty = EnumValue::new("CAT_TREE".to_string());
        enum_ty.description("Top bunk of a cat tree.".to_string());
        assert_eq!(
            enum_ty.to_string(),
            r#"  "Top bunk of a cat tree."
  CAT_TREE"#
        );
    }

    #[test]
    fn it_encodes_an_enum_value_with_directive() {
        let mut enum_ty = EnumValue::new("CARDBOARD_BOX".to_string());
        let mut directive = Directive::new(String::from("testDirective"));
        directive.arg(Argument::new(
            String::from("first"),
            Value::List(vec![Value::Int(1), Value::Int(2)]),
        ));
        enum_ty.description("Box nap\nspot.".to_string());
        enum_ty.directive(directive);

        assert_eq!(
            enum_ty.to_string(),
            r#"  """
  Box nap
  spot.
  """
  CARDBOARD_BOX @testDirective(first: [1, 2])"#
        );
    }

    #[test]
    fn it_encodes_an_enum_value_with_deprecated_block_string_value() {
        let mut enum_ty = EnumValue::new("CARDBOARD_BOX".to_string());
        enum_ty.description("Box nap\nspot.".to_string());
        let mut deprecated_directive = Directive::new(String::from("deprecated"));
        deprecated_directive.arg(Argument::new(
            String::from("reason"),
            Value::String(String::from(r#"Box was "recycled"."#)),
        ));
        enum_ty.directive(deprecated_directive);

        assert_eq!(
            enum_ty.to_string(),
            r#"  """
  Box nap
  spot.
  """
  CARDBOARD_BOX @deprecated(reason: """Box was "recycled".""")"#
        );
    }
}
