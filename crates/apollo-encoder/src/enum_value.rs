use std::fmt;

use crate::StringValue;

/// The __EnumValue type represents one of possible values of an enum.
///
/// *EnumValueDefinition*:
///     Description? EnumValue Directives?
///
/// Detailed documentation can be found in [GraphQL spec](https://spec.graphql.org/October2021/#sec-The-__EnumValue-Type).
///
/// ### Example
/// ```rust
/// use apollo_encoder::{EnumValue};
///
/// let mut enum_value = EnumValue::new("CARDBOARD_BOX");
/// enum_value.description("Box nap spot.");
/// enum_value.deprecated("Box was recycled.");
///
/// assert_eq!(
///     enum_value.to_string(),
///     r#"  "Box nap spot."
///   CARDBOARD_BOX @deprecated(reason: "Box was recycled.")"#
/// );
/// ```
#[derive(Debug, PartialEq, Clone)]
pub struct EnumValue {
    // Name must return a String.
    name: String,
    // Description may return a String or null.
    description: StringValue,
    // Deprecated returns true if this enum value should no longer be used, otherwise false.
    is_deprecated: bool,
    // Deprecation reason optionally provides a reason why this enum value is deprecated.
    deprecation_reason: StringValue,
}

impl EnumValue {
    /// Create a new instance of EnumValue.
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            is_deprecated: false,
            description: StringValue::Field { source: None },
            deprecation_reason: StringValue::Reason { source: None },
        }
    }

    /// Set the Enum Value's description.
    pub fn description(&mut self, description: &str) {
        self.description = StringValue::Field {
            source: Some(description.to_string()),
        };
    }

    /// Set the Enum Value's deprecation properties.
    pub fn deprecated(&mut self, reason: &str) {
        self.is_deprecated = true;
        self.deprecation_reason = StringValue::Reason {
            source: Some(reason.to_string()),
        };
    }
}

impl fmt::Display for EnumValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.description)?;
        write!(f, "  {}", self.name)?;

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
    fn it_encodes_an_enum_value() {
        let enum_value = EnumValue::new("CAT_TREE");

        assert_eq!(enum_value.to_string(), "  CAT_TREE");
    }

    #[test]
    fn it_encodes_an_enum_value_with_desciption() {
        let enum_value = {
            let mut enum_value = EnumValue::new("CAT_TREE");
            enum_value.description("Top bunk of a cat tree.");
            enum_value
        };

        assert_eq!(
            enum_value.to_string(),
            r#"  "Top bunk of a cat tree."
  CAT_TREE"#
        );
    }
    #[test]
    fn it_encodes_an_enum_value_with_deprecated() {
        let enum_value = {
            let mut enum_value = EnumValue::new("CARDBOARD_BOX");
            enum_value.description("Box nap\nspot.");
            enum_value.deprecated("Box was recycled.");
            enum_value
        };

        assert_eq!(
            enum_value.to_string(),
            r#"  """
  Box nap
  spot.
  """
  CARDBOARD_BOX @deprecated(reason: "Box was recycled.")"#
        );
    }

    #[test]
    fn it_encodes_an_enum_value_with_deprecated_block_string_value() {
        let enum_value = {
            let mut enum_value = EnumValue::new("CARDBOARD_BOX");
            enum_value.description("Box nap\nspot.");
            enum_value.deprecated("Box was \"recycled\".");
            enum_value
        };

        assert_eq!(
            enum_value.to_string(),
            r#"  """
  Box nap
  spot.
  """
  CARDBOARD_BOX @deprecated(reason:
  """
  Box was "recycled".
  """
  )"#
        );
    }
}
