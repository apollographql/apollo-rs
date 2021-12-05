use std::fmt;

use crate::{EnumValue, StringValue};

/// Enums are special scalars that can only have a defined set of values.
///
/// *EnumTypeDefinition*:
///     Description? **enum** Name Directives? EnumValuesDefinition?
///
/// Detailed documentation can be found in [GraphQL spec](https://spec.graphql.org/October2021/#sec-Enums).
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

/// ### Example
/// ```rust
/// use apollo_encoder::{EnumValueBuilder, EnumDefBuilder};
///
/// let mut enum_ty_1 = EnumValueBuilder::new("CAT_TREE")
///     .description("Top bunk of a cat tree.")
///     .build();
/// let enum_ty_2 = EnumValueBuilder::new("BED").build();
/// let mut enum_ty_3 = EnumValueBuilder::new("CARDBOARD_BOX")
///     .deprecated("Box was recycled.")
///     .build();
///
/// let enum_ = EnumDefBuilder::new("NapSpots")
///     .description("Favourite cat nap spots.")
///     .value(enum_ty_1)
///     .value(enum_ty_2)
///     .value(enum_ty_3)
///     .build();
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
#[derive(Debug, Clone)]
pub struct EnumDefBuilder {
    // Name must return a String.
    name: String,
    // Description may return a String or null.
    description: Option<String>,
    // A vector of EnumValue. There must be at least one and they must have
    // unique names.
    values: Vec<EnumValue>,
}

impl EnumDefBuilder {
    /// Create a new instance of EnumDefBuilder.
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            description: None,
            values: Vec::new(),
        }
    }

    /// Set the Enum Definition's description.
    pub fn description(mut self, description: &str) -> Self {
        self.description = Some(description.to_string());
        self
    }

    /// Set the Enum Definitions's values.
    pub fn value(mut self, value: EnumValue) -> Self {
        self.values.push(value);
        self
    }

    /// Create a new instance of EnumDef.
    pub fn build(self) -> EnumDef {
        EnumDef {
            name: self.name,
            description: StringValue::Top {
                source: self.description,
            },
            values: self.values,
        }
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
    use crate::{EnumDefBuilder, EnumValueBuilder};
    use pretty_assertions::assert_eq;

    #[test]
    fn it_encodes_a_simple_enum() {
        let enum_ = EnumDefBuilder::new("NapSpots")
            .value(EnumValueBuilder::new("CAT_TREE").build())
            .value(EnumValueBuilder::new("BED").build())
            .value(EnumValueBuilder::new("CARDBOARD_BOX").build())
            .build();

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
        let enum_value_1 = EnumValueBuilder::new("CAT_TREE")
            .description("Top bunk of a cat tree.")
            .build();
        let enum_value_2 = EnumValueBuilder::new("BED").build();
        let enum_value_3 = EnumValueBuilder::new("CARDBOARD_BOX")
            .deprecated("Box was recycled.")
            .build();
        let enum_ = EnumDefBuilder::new("NapSpots")
            .description("Favourite cat nap spots.")
            .value(enum_value_1)
            .value(enum_value_2)
            .value(enum_value_3)
            .build();

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
