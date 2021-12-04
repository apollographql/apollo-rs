use std::fmt;

use crate::{FieldStringValue, ReasonStringValue};

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
/// let mut enum_ty = EnumValue::new("CARDBOARD_BOX".to_string());
/// enum_ty.description(Some("Box nap spot.".to_string()));
/// enum_ty.deprecated(Some("Box was recycled.".to_string()));
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
    description: FieldStringValue,
    // Deprecated returns true if this enum value should no longer be used, otherwise false.
    is_deprecated: bool,
    // Deprecation reason optionally provides a reason why this enum value is deprecated.
    deprecation_reason: ReasonStringValue,
}

impl EnumValue {
    /// Create a new instance of EnumValue.
    pub fn new(name: String) -> Self {
        Self {
            name,
            is_deprecated: false,
            description: Default::default(),
            deprecation_reason: Default::default(),
        }
    }

    /// Set the Enum Value's description.
    pub fn description(&mut self, description: Option<String>) {
        self.description = FieldStringValue::new(description);
    }

    /// Set the Enum Value's deprecation properties.
    pub fn deprecated(&mut self, reason: Option<String>) {
        self.is_deprecated = true;
        self.deprecation_reason = ReasonStringValue::new(reason);
    }
}

impl fmt::Display for EnumValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.description)?;
        write!(f, "  {}", self.name)?;

        if self.is_deprecated {
            write!(f, " @deprecated")?;
            if self.deprecation_reason.is_empty() {
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
        let enum_ty = EnumValue::new("CAT_TREE".to_string());
        assert_eq!(enum_ty.to_string(), "  CAT_TREE");
    }

    #[test]
    fn it_encodes_an_enum_value_with_desciption() {
        let mut enum_ty = EnumValue::new("CAT_TREE".to_string());
        enum_ty.description(Some("Top bunk of a cat tree.".to_string()));
        assert_eq!(
            enum_ty.to_string(),
            r#"  "Top bunk of a cat tree."
  CAT_TREE"#
        );
    }
    #[test]
    fn it_encodes_an_enum_value_with_deprecated() {
        let mut enum_ty = EnumValue::new("CARDBOARD_BOX".to_string());
        enum_ty.description(Some("Box nap\nspot.".to_string()));
        enum_ty.deprecated(Some("Box was recycled.".to_string()));

        assert_eq!(
            enum_ty.to_string(),
            r#"  """
  Box nap
  spot.
  """
  CARDBOARD_BOX @deprecated(reason: "Box was recycled.")"#
        );
    }

    #[test]
    fn it_encodes_an_enum_value_with_deprecated_block_string_value() {
        let mut enum_ty = EnumValue::new("CARDBOARD_BOX".to_string());
        enum_ty.description(Some("Box nap\nspot.".to_string()));
        enum_ty.deprecated(Some("Box was \"recycled\".".to_string()));

        assert_eq!(
            enum_ty.to_string(),
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
