use std::fmt::{self, Display};

/// The __EnumValue type represents one of possible values of an enum.
///
/// *EnumValueDefinition*:
///     Description<sub>opt</sub> EnumValue Directives<sub>\[Const\] opt </sub>
///
/// Detailed documentation can be found in [GraphQL spec](https://spec.graphql.org/draft/#sec-The-__EnumValue-Type).
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
///     r#"  """Box nap spot."""
///   CARDBOARD_BOX @deprecated(reason: "Box was recycled.")"#
/// );
/// ```
#[derive(Debug, PartialEq, Clone)]
pub struct EnumValue {
    // Name must return a String.
    name: String,
    // Description may return a String or null.
    description: Option<String>,
    // Deprecated returns true if this enum value should no longer be used, otherwise false.
    is_deprecated: bool,
    // Deprecation reason optionally provides a reason why this enum value is deprecated.
    deprecation_reason: Option<String>,
}

impl EnumValue {
    /// Create a new instance of EnumValue.
    pub fn new(name: String) -> Self {
        Self {
            name,
            is_deprecated: false,
            description: None,
            deprecation_reason: None,
        }
    }

    /// Set the Enum Value's description.
    pub fn description(&mut self, description: Option<String>) {
        self.description = description;
    }

    /// Set the Enum Value's deprecation properties.
    pub fn deprecated(&mut self, reason: Option<String>) {
        self.is_deprecated = true;
        self.deprecation_reason = reason;
    }
}

impl Display for EnumValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(description) = &self.description {
            // We are determing on whether to have description formatted as
            // a multiline comment based on whether or not it already includes a
            // \n.
            match description.contains('\n') {
                true => writeln!(f, "  \"\"\"\n  {}\n  \"\"\"", description)?,
                false => writeln!(f, "  \"\"\"{}\"\"\"", description)?,
            }
        }

        write!(f, "  {}", self.name)?;

        if self.is_deprecated {
            write!(f, " @deprecated")?;
            // Just in case deprecated directive is ever used without a reason,
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
            r#"  """Top bunk of a cat tree."""
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
}
