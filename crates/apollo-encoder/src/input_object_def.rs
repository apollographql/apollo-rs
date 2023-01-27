use std::fmt;

use crate::{Directive, InputField, StringValue};

/// Input objects are composite types used as inputs into queries defined as a list of named input values..
///
/// InputObjectTypeDefinition
///     Description? **input** Name Directives? FieldsDefinition?
///
/// Detailed documentation can be found in [GraphQL spec](https://spec.graphql.org/October2021/#sec-Input-Objects).
///
/// ### Example
/// ```rust
/// use apollo_encoder::{Type_, InputField, InputObjectDefinition};
/// use indoc::indoc;
///
/// let ty_1 = Type_::NamedType {
///     name: "DanglerPoleToys".to_string(),
/// };
///
/// let ty_2 = Type_::List { ty: Box::new(ty_1) };
/// let mut field = InputField::new("toys".to_string(), ty_2);
/// field.default_value("\"Cat Dangler Pole Bird\"".to_string());
/// let ty_3 = Type_::NamedType {
///     name: "FavouriteSpots".to_string(),
/// };
/// let mut field_2 = InputField::new("playSpot".to_string(), ty_3);
/// field_2.description("Best playime spots, e.g. tree, bed.".to_string());
///
/// let mut input_def = InputObjectDefinition::new("PlayTime".to_string());
/// input_def.field(field);
/// input_def.field(field_2);
/// input_def.description("Cat playtime input".to_string());
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
pub struct InputObjectDefinition {
    // Name must return a String.
    name: String,
    // Description may return a String or null.
    description: Option<StringValue>,
    // A vector of fields
    fields: Vec<InputField>,
    /// Contains all directives.
    directives: Vec<Directive>,
    extend: bool,
}

impl InputObjectDefinition {
    /// Create a new instance of ObjectDef with a name.
    pub fn new(name: String) -> Self {
        Self {
            name,
            description: None,
            fields: Vec::new(),
            directives: Vec::new(),
            extend: false,
        }
    }

    /// Set the input object type as an extension
    pub fn extend(&mut self) {
        self.extend = true;
    }

    /// Set the InputObjectDef's description field.
    pub fn description(&mut self, description: String) {
        self.description = Some(StringValue::Top {
            source: description,
        });
    }

    /// Push a Field to InputObjectDef's fields vector.
    pub fn field(&mut self, field: InputField) {
        self.fields.push(field)
    }

    /// Add a directive.
    pub fn directive(&mut self, directive: Directive) {
        self.directives.push(directive)
    }
}

impl fmt::Display for InputObjectDefinition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.extend {
            write!(f, "extend ")?;
        // No description when it's a extension
        } else if let Some(description) = &self.description {
            write!(f, "{description}")?;
        }

        write!(f, "input {}", &self.name)?;

        for directive in &self.directives {
            write!(f, " {directive}")?;
        }
        write!(f, " {{")?;

        for field in &self.fields {
            write!(f, "\n{field}")?;
        }
        writeln!(f, "\n}}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Argument, InputField, Type_, Value};
    use indoc::indoc;
    use pretty_assertions::assert_eq;

    #[test]
    fn it_encodes_input_object() {
        let ty_1 = Type_::NamedType {
            name: "DanglerPoleToys".to_string(),
        };

        let ty_2 = Type_::List { ty: Box::new(ty_1) };
        let mut field = InputField::new("toys".to_string(), ty_2);
        field.default_value("\"Cat Dangler Pole Bird\"".to_string());
        let ty_3 = Type_::NamedType {
            name: "FavouriteSpots".to_string(),
        };
        let mut field_2 = InputField::new("playSpot".to_string(), ty_3);
        field_2.description("Best playime spots, e.g. tree, bed.".to_string());
        let mut directive = Directive::new(String::from("testDirective"));
        directive.arg(Argument::new(
            String::from("first"),
            Value::String("one".to_string()),
        ));

        let mut input_def = InputObjectDefinition::new("PlayTime".to_string());
        input_def.field(field);
        input_def.field(field_2);
        input_def.directive(directive);

        assert_eq!(
            input_def.to_string(),
            indoc! { r#"
                input PlayTime @testDirective(first: "one") {
                  toys: [DanglerPoleToys] = "Cat Dangler Pole Bird"
                  "Best playime spots, e.g. tree, bed."
                  playSpot: FavouriteSpots
                }
            "#}
        );
    }

    #[test]
    fn it_encodes_input_object_with_description() {
        let ty_1 = Type_::NamedType {
            name: "DanglerPoleToys".to_string(),
        };

        let ty_2 = Type_::List { ty: Box::new(ty_1) };
        let mut field = InputField::new("toys".to_string(), ty_2);
        field.default_value("\"Cat Dangler Pole Bird\"".to_string());
        let ty_3 = Type_::NamedType {
            name: "FavouriteSpots".to_string(),
        };
        let mut field_2 = InputField::new("playSpot".to_string(), ty_3);
        field_2.description("Best playime spots, e.g. tree, bed.".to_string());

        let mut input_def = InputObjectDefinition::new("PlayTime".to_string());
        input_def.field(field);
        input_def.field(field_2);
        input_def.description("Cat playtime input".to_string());

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

    #[test]
    fn it_encodes_input_object_extension() {
        let ty_1 = Type_::NamedType {
            name: "DanglerPoleToys".to_string(),
        };

        let ty_2 = Type_::List { ty: Box::new(ty_1) };
        let mut field = InputField::new("toys".to_string(), ty_2);
        field.default_value("\"Cat Dangler Pole Bird\"".to_string());
        let ty_3 = Type_::NamedType {
            name: "FavouriteSpots".to_string(),
        };
        let mut field_2 = InputField::new("playSpot".to_string(), ty_3);
        field_2.description("Best playime spots, e.g. tree, bed.".to_string());

        let mut input_def = InputObjectDefinition::new("PlayTime".to_string());
        input_def.field(field);
        input_def.field(field_2);
        input_def.description("Cat playtime input".to_string());
        input_def.extend();

        assert_eq!(
            input_def.to_string(),
            indoc! { r#"
                extend input PlayTime {
                  toys: [DanglerPoleToys] = "Cat Dangler Pole Bird"
                  "Best playime spots, e.g. tree, bed."
                  playSpot: FavouriteSpots
                }
            "#}
        );
    }
}
