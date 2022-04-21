use std::fmt;

use crate::{Directive, EnumValue, StringValue};

/// Enums are special scalars that can only have a defined set of values.
///
/// *EnumTypeDefinition*:
///     Description? **enum** Name Directives? EnumValuesDefinition?
///
/// Detailed documentation can be found in [GraphQL spec](https://spec.graphql.org/October2021/#sec-Enums).
///
/// ### Example
/// ```rust
/// use apollo_encoder::{Argument, Directive, EnumValue, EnumDefinition, Value};
///
/// let mut enum_ty_1 = EnumValue::new("CAT_TREE".to_string());
/// enum_ty_1.description("Top bunk of a cat tree.".to_string());
/// let enum_ty_2 = EnumValue::new("BED".to_string());
/// let mut deprecated_directive = Directive::new(String::from("deprecated"));
/// deprecated_directive.arg(Argument::new(String::from("reason"), Value::String(String::from("Box was recycled."))));
/// let mut enum_ty_3 = EnumValue::new("CARDBOARD_BOX".to_string());
/// enum_ty_3.directive(deprecated_directive);
///
/// let mut enum_ = EnumDefinition::new("NapSpots".to_string());
/// enum_.description("Favourite cat nap spots.".to_string());
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
pub struct EnumDefinition {
    // Name must return a String.
    name: String,
    // Description may return a String or null.
    description: Option<StringValue>,
    // A vector of EnumValue. There must be at least one and they must have
    // unique names.
    values: Vec<EnumValue>,
    /// The vector of directives
    directives: Vec<Directive>,
    extend: bool,
}

impl EnumDefinition {
    /// Create a new instance of Enum Definition.
    pub fn new(name: String) -> Self {
        Self {
            name,
            description: None,
            values: Vec::new(),
            directives: Vec::new(),
            extend: false,
        }
    }

    /// Set the enum type as an extension
    pub fn extend(&mut self) {
        self.extend = true;
    }

    /// Set the Enum Definition's description.
    pub fn description(&mut self, description: String) {
        self.description = Some(StringValue::Top {
            source: description,
        });
    }

    /// Set the Enum Definitions's values.
    pub fn value(&mut self, value: EnumValue) {
        self.values.push(value)
    }

    /// Add a directive.
    pub fn directive(&mut self, directive: Directive) {
        self.directives.push(directive)
    }
}

impl fmt::Display for EnumDefinition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.extend {
            write!(f, "extend ")?;
        // No description when it's a extension
        } else if let Some(description) = &self.description {
            write!(f, "{}", description)?;
        }

        write!(f, "enum {}", self.name)?;
        for directive in &self.directives {
            write!(f, " {}", directive)?;
        }

        write!(f, " {{")?;
        for value in &self.values {
            write!(f, "\n{}", value)?;
        }
        writeln!(f, "\n}}")
    }
}

#[cfg(test)]
mod tests {
    use crate::{Argument, Value};

    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn it_encodes_a_simple_enum() {
        let enum_ty_1 = EnumValue::new("CAT_TREE".to_string());
        let enum_ty_2 = EnumValue::new("BED".to_string());
        let enum_ty_3 = EnumValue::new("CARDBOARD_BOX".to_string());

        let mut enum_ = EnumDefinition::new("NapSpots".to_string());
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
        enum_ty_1.description("Top bunk of a cat tree.".to_string());
        let enum_ty_2 = EnumValue::new("BED".to_string());
        let mut enum_ty_3 = EnumValue::new("CARDBOARD_BOX".to_string());
        let mut deprecated_directive = Directive::new(String::from("deprecated"));
        deprecated_directive.arg(Argument::new(
            String::from("reason"),
            Value::String(String::from("Box was recycled.")),
        ));
        enum_ty_3.directive(deprecated_directive);
        let mut directive = Directive::new(String::from("testDirective"));
        directive.arg(Argument::new(
            String::from("first"),
            Value::String("one".to_string()),
        ));

        let mut enum_ = EnumDefinition::new("NapSpots".to_string());
        enum_.description("Favourite cat nap spots.".to_string());
        enum_.value(enum_ty_1);
        enum_.value(enum_ty_2);
        enum_.value(enum_ty_3);
        enum_.directive(directive);

        assert_eq!(
            enum_.to_string(),
            r#""Favourite cat nap spots."
enum NapSpots @testDirective(first: "one") {
  "Top bunk of a cat tree."
  CAT_TREE
  BED
  CARDBOARD_BOX @deprecated(reason: "Box was recycled.")
}
"#
        );
    }

    #[test]
    fn it_encodes_enum_extension() {
        let mut enum_ty_1 = EnumValue::new("CAT_TREE".to_string());
        enum_ty_1.description("Top bunk of a cat tree.".to_string());
        let enum_ty_2 = EnumValue::new("BED".to_string());
        let mut enum_ty_3 = EnumValue::new("CARDBOARD_BOX".to_string());
        let mut deprecated_directive = Directive::new(String::from("deprecated"));
        deprecated_directive.arg(Argument::new(
            String::from("reason"),
            Value::String(String::from("Box was recycled.")),
        ));
        enum_ty_3.directive(deprecated_directive);
        let mut directive = Directive::new(String::from("testDirective"));
        directive.arg(Argument::new(
            String::from("first"),
            Value::String("one".to_string()),
        ));

        let mut enum_ = EnumDefinition::new("NapSpots".to_string());
        enum_.description("Favourite cat nap spots.".to_string());
        enum_.value(enum_ty_1);
        enum_.value(enum_ty_2);
        enum_.value(enum_ty_3);
        enum_.directive(directive);
        enum_.extend();

        assert_eq!(
            enum_.to_string(),
            r#"extend enum NapSpots @testDirective(first: "one") {
  "Top bunk of a cat tree."
  CAT_TREE
  BED
  CARDBOARD_BOX @deprecated(reason: "Box was recycled.")
}
"#
        );
    }
}
