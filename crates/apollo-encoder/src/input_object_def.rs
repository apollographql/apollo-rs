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
#[derive(Debug, Clone)]
pub struct InputObjectDef {
    // Name must return a String.
    name: String,
    // Description may return a String or null.
    description: StringValue,
    // A vector of fields
    fields: Vec<InputField>,
}

/// ### Example
/// ```rust
/// use apollo_encoder::{Type_, InputFieldBuilder, InputObjectDefBuilder};
/// use indoc::indoc;
///
/// let field_1 = {
///     let ty = Type_::list(Type_::named("DanglerPoleToys"));
///
///     InputFieldBuilder::new("toys", ty)
///         .default("\"Cat Dangler Pole Bird\"")
///         .build()
/// };
/// let field_2 = {
///     let ty = Type_::named("FavouriteSpots");
///
///     InputFieldBuilder::new("playSpot", ty)
///         .description("Best playime spots, e.g. tree, bed.")
///         .build()
/// };
///
/// let input_def = InputObjectDefBuilder::new("PlayTime")
///     .field(field_1)
///     .field(field_2)
///     .description("Cat playtime input")
///     .build();
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
    use crate::{InputFieldBuilder, InputObjectDefBuilder, Type_};
    use indoc::indoc;
    use pretty_assertions::assert_eq;

    #[test]
    fn it_encodes_input_object() {
        let field_1 = {
            let ty = Type_::list(Type_::named("DanglerPoleToys"));

            InputFieldBuilder::new("toys", ty)
                .default("\"Cat Dangler Pole Bird\"")
                .build()
        };

        let field_2 = {
            let ty = Type_::named("FavouriteSpots");

            InputFieldBuilder::new("playSpot", ty)
                .description("Best playime spots, e.g. tree, bed.")
                .build()
        };

        let input_def = InputObjectDefBuilder::new("PlayTime")
            .field(field_1)
            .field(field_2)
            .build();

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
            let ty = Type_::list(Type_::named("DanglerPoleToys"));

            InputFieldBuilder::new("toys", ty)
                .default("\"Cat Dangler Pole Bird\"")
                .build()
        };

        let field_2 = {
            let ty = Type_::named("FavouriteSpots");

            InputFieldBuilder::new("playSpot", ty)
                .description("Best playime spots, e.g. tree, bed.")
                .build()
        };

        let input_def = InputObjectDefBuilder::new("PlayTime")
            .field(field_1)
            .field(field_2)
            .description("Cat playtime input")
            .build();

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
