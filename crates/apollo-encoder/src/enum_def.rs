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
/// let mut enum_ty_1 = EnumValue::new("CAT_TREE");
/// enum_ty_1.description("Top bunk of a cat tree.");
/// let enum_ty_2 = EnumValue::new("BED");
/// let mut enum_ty_3 = EnumValue::new("CARDBOARD_BOX");
/// enum_ty_3.deprecated("Box was recycled.");
///
/// let mut enum_ = EnumDef::new("NapSpots");
/// enum_.description("Favourite cat nap spots.");
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
            name: name.to_string(),
            description: StringValue::Top { source: None },
            values: Vec::new(),
        }
    }

    /// Set the Enum Definition's description.
    pub fn description(&mut self, description: &str) {
        self.description = StringValue::Top {
            source: Some(description.to_string()),
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
        let enum_ = {
            let mut enum_ = EnumDef::new("NapSpots");
            enum_.value(EnumValue::new("CAT_TREE"));
            enum_.value(EnumValue::new("BED"));
            enum_.value(EnumValue::new("CARDBOARD_BOX"));
            enum_
        };

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
        let enum_ = {
            let enum_value_1 = {
                let mut enum_value = EnumValue::new("CAT_TREE");
                enum_value.description("Top bunk of a cat tree.");
                enum_value
            };
            let enum_value_2 = EnumValue::new("BED");
            let enum_value_3 = {
                let mut enum_value = EnumValue::new("CARDBOARD_BOX");
                enum_value.deprecated("Box was recycled.");
                enum_value
            };

            let mut enum_ = EnumDef::new("NapSpots");
            enum_.description("Favourite cat nap spots.");
            enum_.value(enum_value_1);
            enum_.value(enum_value_2);
            enum_.value(enum_value_3);
            enum_
        };

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
