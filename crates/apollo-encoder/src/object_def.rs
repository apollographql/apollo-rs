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

/// ### Example
/// ```rust
/// use apollo_encoder::{Type_, FieldBuilder, ObjectDefBuilder};
/// use indoc::indoc;
///
/// let field_1 = {
///     let ty = Type_::named_type("DanglerPoleToys");
///     let ty = Type_::list(Box::new(ty));
///
///     FieldBuilder::new("toys", ty)
///         .deprecated("Cats are too spoiled")
///         .build()
/// };
/// let field_2 = {
///     let ty = Type_::named_type("FoodType");
///
///     FieldBuilder::new("food", ty)
///         .description("Dry or wet food?")
///         .build()
/// };
///
/// let field_3 = {
///     let ty = Type_::named_type("Boolean");
///
///     FieldBuilder::new("catGrass", ty).build()
/// };
///
/// let object_def = ObjectDefBuilder::new("PetStoreTrip")
///     .field(field_1)
///     .field(field_2)
///     .field(field_3)
///     .interface("ShoppingTrip")
///     .build();
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
pub struct ObjectDefBuilder {
    // Name must return a String.
    name: String,
    // Description may return a String or null.
    description: Option<String>,
    // The vector of interfaces that an object implements.
    interfaces: Vec<String>,
    // The vector of fields query‐able on this type.
    fields: Vec<Field>,
}

impl ObjectDefBuilder {
    /// Create a new instance of ObjectDef with a name.
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            description: None,
            interfaces: Vec::new(),
            fields: Vec::new(),
        }
    }

    /// Set the ObjectDef's description field.
    pub fn description(mut self, description: &str) -> Self {
        self.description = Some(description.to_string());
        self
    }

    /// Set the interfaces ObjectDef implements.
    pub fn interface(mut self, interface: &str) -> Self {
        self.interfaces.push(interface.to_string());
        self
    }

    /// Push a Field to ObjectDef's fields vector.
    pub fn field(mut self, field: Field) -> Self {
        self.fields.push(field);
        self
    }

    /// Create a new instance of ObjectDef with a name.
    pub fn build(self) -> ObjectDef {
        ObjectDef {
            name: self.name,
            description: StringValue::Top {
                source: self.description,
            },
            interfaces: self.interfaces,
            fields: self.fields,
        }
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
    use crate::{FieldBuilder, ObjectDefBuilder, Type_};
    use indoc::indoc;
    use pretty_assertions::assert_eq;

    #[test]
    fn it_encodes_object_with_description() {
        let field = {
            let ty = Type_::named_type("DanglerPoleToys");
            let ty = Type_::list(Box::new(ty));

            FieldBuilder::new("toys", ty).build()
        };

        let object_def = ObjectDefBuilder::new("PetStoreTrip")
            .field(field)
            .description("What to get at Fressnapf?")
            .build();

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

            FieldBuilder::new("toys", ty)
                .deprecated("\"DanglerPoleToys\" are no longer interesting")
                .build()
        };

        let object_def = ObjectDefBuilder::new("PetStoreTrip")
            .field(field)
            .description("What to get at Fressnapf?")
            .build();

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

            FieldBuilder::new("toys", ty)
                .deprecated("Cats are too spoiled")
                .build()
        };

        let field_2 = {
            let ty = Type_::named_type("FoodType");

            FieldBuilder::new("food", ty)
                .description("Dry or wet food?")
                .build()
        };

        let field_3 = {
            let ty = Type_::named_type("Boolean");

            FieldBuilder::new("catGrass", ty).build()
        };

        let object_def = ObjectDefBuilder::new("PetStoreTrip")
            .field(field_1)
            .field(field_2)
            .field(field_3)
            .description("Shopping list for cats at the pet store.")
            .interface("ShoppingTrip")
            .build();

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

            FieldBuilder::new("name", ty)
                .description("multiline\ndescription")
                .build()
        };

        let object_def = ObjectDefBuilder::new("Book")
            .field(field)
            .description("Book Object\nType")
            .build();

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
