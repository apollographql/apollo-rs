use std::fmt;

use crate::{InputField, StringValue};

/// Input objects are composite types used as inputs into queries defined as a list of named input values..
///
/// InputObjectTypeDefinition
///     Description? **input** Name Directives? FieldsDefinition?
///
/// Detailed documentation can be found in [GraphQL spec](https://spec.graphql.org/October2021/#sec-Input-Objects).
///
/// **Note**: At the moment InputObjectTypeDefinition differs slightly from the
/// spec. Instead of accepting InputValues as `field` parameter, we accept
/// InputField.
///
/// ### Example
/// ```rust
/// use apollo_encoder::{Type_, InputField, InputObjectDef};
/// use indoc::indoc;
///
/// let ty_1 = Type_::named_type("DanglerPoleToys");
///
/// let ty_2 = Type_::list(Box::new(ty_1));
/// let mut field = InputField::new("toys", ty_2);
/// field.default("\"Cat Dangler Pole Bird\"");
/// let ty_3 = Type_::named_type("FavouriteSpots");
/// let mut field_2 = InputField::new("playSpot", ty_3);
/// field_2.description("Best playime spots, e.g. tree, bed.");
///
/// let mut input_def = InputObjectDef::new("PlayTime");
/// input_def.field(field);
/// input_def.field(field_2);
/// input_def.description("Cat playtime input");
///
/// assert_eq!(
///     input_def.to_string(),
///     indoc! { r#"
///         "Cat playtime input"
///         input PlayTime {
///           toys: [DanglerPoleToys] = "Cat Dangler Pole Bird"
///           "Best playime spots, e.g. tree, bed."
///           playSpot: FavouriteSpots
///         }
///     "#}
/// );
/// ```
#[derive(Debug, Clone)]
pub struct InputObjectDef {
    // Name must return a String.
    name: String,
    // Description may return a String or null.
    description: StringValue,
    // A vector of fields
    fields: Vec<InputField>,
}

#[derive(Debug, Clone)]
pub struct InputObjectDefBuilder {
    // Name must return a String.
    name: String,
    // Description may return a String or null.
    description: Option<String>,
    // A vector of fields
    fields: Vec<InputField>,
}

impl InputObjectDefBuilder {
    /// Create a new instance of ObjectDefBuilder.
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            description: None,
            fields: Vec::new(),
        }
    }

    /// Set the InputObjectDef's description field.
    pub fn description(mut self, description: &str) -> Self {
        self.description = Some(description.to_string());
        self
    }

    /// Push a Field to InputObjectDef's fields vector.
    pub fn field(mut self, field: InputField) -> Self {
        self.fields.push(field);
        self
    }

    /// Create a new instance of ObjectDef.
    pub fn build(self) -> InputObjectDef {
        InputObjectDef {
            name: self.name,
            description: StringValue::Top {
                source: self.description,
            },
            fields: self.fields,
        }
    }
}

impl fmt::Display for InputObjectDef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.description)?;

        write!(f, "input {} {{", &self.name)?;

        for field in &self.fields {
            write!(f, "\n{}", field)?;
        }
        writeln!(f, "\n}}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{InputField, Type_};
    use indoc::indoc;
    use pretty_assertions::assert_eq;

    #[test]
    fn it_encodes_input_object() {
        let field_1 = {
            let ty = Type_::named_type("DanglerPoleToys");
            let ty = Type_::list(Box::new(ty));

            let mut field = InputField::new("toys", ty);
            field.default("\"Cat Dangler Pole Bird\"");
            field
        };

        let field_2 = {
            let ty = Type_::named_type("FavouriteSpots");

            let mut field = InputField::new("playSpot", ty);
            field.description("Best playime spots, e.g. tree, bed.");
            field
        };

        let input_def = {
            let mut input_def = InputObjectDef::new("PlayTime");
            input_def.field(field_1);
            input_def.field(field_2);
            input_def
        };

        assert_eq!(
            input_def.to_string(),
            indoc! { r#"
                input PlayTime {
                  toys: [DanglerPoleToys] = "Cat Dangler Pole Bird"
                  "Best playime spots, e.g. tree, bed."
                  playSpot: FavouriteSpots
                }
            "#}
        );
    }

    #[test]
    fn it_encodes_input_object_with_description() {
        let field_1 = {
            let ty = Type_::named_type("DanglerPoleToys");
            let ty = Type_::list(Box::new(ty));

            let mut field = InputField::new("toys", ty);
            field.default("\"Cat Dangler Pole Bird\"");
            field
        };

        let field_2 = {
            let ty = Type_::named_type("FavouriteSpots");

            let mut field = InputField::new("playSpot", ty);
            field.description("Best playime spots, e.g. tree, bed.");
            field
        };

        let input_def = {
            let mut input_def = InputObjectDef::new("PlayTime");
            input_def.field(field_1);
            input_def.field(field_2);
            input_def.description("Cat playtime input");
            input_def
        };

        assert_eq!(
            input_def.to_string(),
            indoc! { r#"
                "Cat playtime input"
                input PlayTime {
                  toys: [DanglerPoleToys] = "Cat Dangler Pole Bird"
                  "Best playime spots, e.g. tree, bed."
                  playSpot: FavouriteSpots
                }
            "#}
        );
    }
}
