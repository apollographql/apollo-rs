use std::fmt;

use crate::{EnumValue, StringValue};

/// Enums are special scalars that can only have a defined set of values.
///
/// *EnumTypeDefinition*:
///     Description? **enum** Name Directives? EnumValuesDefinition?
///
/// Detailed documentation can be found in [GraphQL spec](https://spec.graphql.org/October2021/#sec-Enums).
///
/// ### Example
/// ```rust
/// use apollo_encoder::{EnumValue, EnumDef};
///
/// let mut enum_ty_1 = EnumValue::new("CAT_TREE".to_string());
/// enum_ty_1.description(Some("Top bunk of a cat tree.".to_string()));
/// let enum_ty_2 = EnumValue::new("BED".to_string());
/// let mut enum_ty_3 = EnumValue::new("CARDBOARD_BOX".to_string());
/// enum_ty_3.deprecated(Some("Box was recycled.".to_string()));
///
/// let mut enum_ = EnumDef::new("NapSpots".to_string());
/// enum_.description(Some("Favourite cat nap spots.".to_string()));
/// enum_.value(enum_ty_1);
/// enum_.value(enum_ty_2);
/// enum_.value(enum_ty_3);
///
/// assert_eq!(
///     enum_.to_string(),
///     r#""Favourite cat nap spots."
/// enum NapSpots {
///   "Top bunk of a cat tree."
///   CAT_TREE
///   BED
///   CARDBOARD_BOX @deprecated(reason: "Box was recycled.")
/// }
/// "#
/// );
/// ```
#[derive(Debug, PartialEq, Clone)]
pub struct EnumDef {
    // Name must return a String.
    name: String,
    // Description may return a String or null.
    description: StringValue,
    // A vector of EnumValue. There must be at least one and they must have
    // unique names.
    values: Vec<EnumValue>,
}

impl EnumDef {
    /// Create a new instance of Enum Definition.
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_owned(),
            description: StringValue::Top { source: None },
            values: Vec::new(),
        }
    }

    /// Set the Enum Definition's description.
    pub fn description(&mut self, description: &str) {
        self.description = StringValue::Top {
            source: Some(description.to_owned()),
        };
    }

    /// Set the Enum Definitions's values.
    pub fn value(&mut self, value: EnumValue) {
        self.values.push(value)
    }
}

impl fmt::Display for EnumDef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.description)?;
        write!(f, "enum {} {{", self.name)?;
        for value in &self.values {
            write!(f, "\n{}", value)?;
        }
        writeln!(f, "\n}}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn it_encodes_a_simple_enum() {
        let enum_ty_1 = EnumValue::new("CAT_TREE".to_string());
        let enum_ty_2 = EnumValue::new("BED".to_string());
        let enum_ty_3 = EnumValue::new("CARDBOARD_BOX".to_string());

        let mut enum_ = EnumDef::new("NapSpots".to_string());
        enum_.value(enum_ty_1);
        enum_.value(enum_ty_2);
        enum_.value(enum_ty_3);

        assert_eq!(
            enum_.to_string(),
            r#"enum NapSpots {
  CAT_TREE
  BED
  CARDBOARD_BOX
}
"#
        );
    }
    #[test]
    fn it_encodes_enum_with_descriptions() {
        let mut enum_ty_1 = EnumValue::new("CAT_TREE".to_string());
        enum_ty_1.description(Some("Top bunk of a cat tree.".to_string()));
        let enum_ty_2 = EnumValue::new("BED".to_string());
        let mut enum_ty_3 = EnumValue::new("CARDBOARD_BOX".to_string());
        enum_ty_3.deprecated(Some("Box was recycled.".to_string()));

        let mut enum_ = EnumDef::new("NapSpots".to_string());
        enum_.description(Some("Favourite cat nap spots.".to_string()));
        enum_.value(enum_ty_1);
        enum_.value(enum_ty_2);
        enum_.value(enum_ty_3);

        assert_eq!(
            enum_.to_string(),
            r#""Favourite cat nap spots."
enum NapSpots {
  "Top bunk of a cat tree."
  CAT_TREE
  BED
  CARDBOARD_BOX @deprecated(reason: "Box was recycled.")
}
"#
        );
    }
}
