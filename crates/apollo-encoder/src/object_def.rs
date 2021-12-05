use std::fmt;

use crate::{Field, StringValue};
/// Object types represent concrete instantiations of sets of fields.
///
/// The introspection types (e.g. `__Type`, `__Field`, etc) are examples of
/// objects.
///
/// *ObjectTypeDefinition*:
///     Description? **type** Name ImplementsInterfaces? Directives? FieldsDefinition?
///
/// Detailed documentation can be found in [GraphQL spec](https://spec.graphql.org/October2021/#sec-Object).
///
/// ### Example
/// ```rust
/// use apollo_encoder::{Type_, Field, ObjectDef};
/// use indoc::indoc;
///
/// let ty_1 = Type_::named_type("DanglerPoleToys");
///
/// let ty_2 = Type_::list(Box::new(ty_1));
/// let mut field = Field::new("toys", ty_2);
/// field.deprecated("Cats are too spoiled");
/// let ty_3 = Type_::named_type("FoodType");
/// let mut field_2 = Field::new("food", ty_3);
/// field_2.description("Dry or wet food?");
///
/// let ty_4 = Type_::named_type("Boolean");
/// let field_3 = Field::new("catGrass", ty_4);
///
/// let mut object_def = ObjectDef::new("PetStoreTrip");
/// object_def.field(field);
/// object_def.field(field_2);
/// object_def.field(field_3);
/// object_def.interface("ShoppingTrip");
///
/// assert_eq!(
///     object_def.to_string(),
///     indoc! { r#"
///         type PetStoreTrip implements ShoppingTrip {
///           toys: [DanglerPoleToys] @deprecated(reason: "Cats are too spoiled")
///           "Dry or wet food?"
///           food: FoodType
///           catGrass: Boolean
///         }
///     "#}
/// );
/// ```
#[derive(Debug)]
pub struct ObjectDef {
    // Name must return a String.
    name: String,
    // Description may return a String or null.
    description: StringValue,
    // The vector of interfaces that an object implements.
    interfaces: Vec<String>,
    // The vector of fields query‐able on this type.
    fields: Vec<Field>,
}

impl ObjectDef {
    /// Create a new instance of ObjectDef with a name.
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            description: StringValue::Top { source: None },
            interfaces: Vec::new(),
            fields: Vec::new(),
        }
    }

    /// Set the ObjectDef's description field.
    pub fn description(&mut self, description: &str) {
        self.description = StringValue::Top {
            source: Some(description.to_string()),
        };
    }

    /// Set the interfaces ObjectDef implements.
    pub fn interface(&mut self, interface: &str) {
        self.interfaces.push(interface.to_string())
    }

    /// Push a Field to ObjectDef's fields vector.
    pub fn field(&mut self, field: Field) {
        self.fields.push(field)
    }
}

impl fmt::Display for ObjectDef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.description)?;

        write!(f, "type {}", &self.name)?;
        for (i, interface) in self.interfaces.iter().enumerate() {
            match i {
                0 => write!(f, " implements {}", interface)?,
                _ => write!(f, "& {}", interface)?,
            }
        }
        write!(f, " {{")?;

        for field in &self.fields {
            write!(f, "\n{}", field)?;
        }
        writeln!(f, "\n}}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Field, Type_};
    use indoc::indoc;
    use pretty_assertions::assert_eq;

    #[test]
    fn it_encodes_object_with_description() {
        let field = {
            let ty = Type_::named_type("DanglerPoleToys");
            let ty = Type_::list(Box::new(ty));

            Field::new("toys", ty)
        };

        let object_def = {
            let mut object_def = ObjectDef::new("PetStoreTrip");
            object_def.field(field);
            object_def.description("What to get at Fressnapf?");
            object_def
        };

        assert_eq!(
            object_def.to_string(),
            indoc! { r#"
                "What to get at Fressnapf?"
                type PetStoreTrip {
                  toys: [DanglerPoleToys]
                }
            "#}
        );
    }

    #[test]
    fn it_encodes_object_with_field_directives() {
        let field = {
            let ty = Type_::named_type("DanglerPoleToys");

            let mut field = Field::new("toys", ty);
            field.deprecated("\"DanglerPoleToys\" are no longer interesting");
            field
        };

        let object_def = {
            let mut object_def = ObjectDef::new("PetStoreTrip");
            object_def.field(field);
            object_def.description("What to get at Fressnapf?");
            object_def
        };

        assert_eq!(
            object_def.to_string(),
            indoc! { r#"
                "What to get at Fressnapf?"
                type PetStoreTrip {
                  toys: DanglerPoleToys @deprecated(reason:
                  """
                  "DanglerPoleToys" are no longer interesting
                  """
                  )
                }
            "#}
        );
    }

    #[test]
    fn it_encodes_object_with_interface() {
        let field_1 = {
            let ty = Type_::named_type("DanglerPoleToys");
            let ty = Type_::list(Box::new(ty));

            let mut field = Field::new("toys", ty);
            field.deprecated("Cats are too spoiled");
            field
        };

        let field_2 = {
            let ty = Type_::named_type("FoodType");

            let mut field = Field::new("food", ty);
            field.description("Dry or wet food?");
            field
        };

        let field_3 = {
            let ty = Type_::named_type("Boolean");
            Field::new("catGrass", ty)
        };

        let object_def = {
            let mut object_def = ObjectDef::new("PetStoreTrip");
            object_def.field(field_1);
            object_def.field(field_2);
            object_def.field(field_3);
            object_def.description("Shopping list for cats at the pet store.");
            object_def.interface("ShoppingTrip");
            object_def
        };

        assert_eq!(
            object_def.to_string(),
            indoc! { r#"
                "Shopping list for cats at the pet store."
                type PetStoreTrip implements ShoppingTrip {
                  toys: [DanglerPoleToys] @deprecated(reason: "Cats are too spoiled")
                  "Dry or wet food?"
                  food: FoodType
                  catGrass: Boolean
                }
            "#}
        );
    }

    #[test]
    fn it_encodes_object_with_block_string_description() {
        let field = {
            let ty = Type_::named_type("String");

            let mut field = Field::new("name", ty);
            field.description("multiline\ndescription");
            field
        };

        let object_def = {
            let mut object_def = ObjectDef::new("Book");
            object_def.field(field);
            object_def.description("Book Object\nType");
            object_def
        };

        assert_eq!(
            object_def.to_string(),
            indoc! { r#"
                """
                Book Object
                Type
                """
                type Book {
                  """
                  multiline
                  description
                  """
                  name: String
                }
            "#}
        );
    }
}
