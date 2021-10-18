use std::fmt;

use crate::Field;
/// Object types represent concrete instantiations of sets of fields.
///
/// The introspection types (e.g. `__Type`, `__Field`, etc) are examples of
/// objects.
///
/// *ObjectTypeDefinition*:
///     Description<sub>opt</sub> **type** Name ImplementsInterfaces<sub>opt</sub> Directives<sub>\[Const\] opt</sub> FieldsDefinition<sub>opt</sub>
///
/// Detailed documentation can be found in [GraphQL spec](https://spec.graphql.org/draft/#sec-Object).
///
/// ### Example
/// ```rust
/// use apollo_encoder::{Type_, Field, ObjectDef};
/// use indoc::indoc;
///
/// let ty_1 = Type_::NamedType {
///     name: "DanglerPoleToys".to_string(),
/// };
///
/// let ty_2 = Type_::List { ty: Box::new(ty_1) };
/// let mut field = Field::new("toys".to_string(), ty_2);
/// field.deprecated(Some("Cats are too spoiled".to_string()));
/// let ty_3 = Type_::NamedType {
///     name: "FoodType".to_string(),
/// };
/// let mut field_2 = Field::new("food".to_string(), ty_3);
/// field_2.description(Some("Dry or wet food?".to_string()));
///
/// let ty_4 = Type_::NamedType {
///     name: "Boolean".to_string(),
/// };
/// let field_3 = Field::new("catGrass".to_string(), ty_4);
///
/// let mut object_def = ObjectDef::new("PetStoreTrip".to_string());
/// object_def.field(field);
/// object_def.field(field_2);
/// object_def.field(field_3);
/// object_def.interface("ShoppingTrip".to_string());
///
/// assert_eq!(
///     object_def.to_string(),
///     indoc! { r#"
///         type PetStoreTrip implements ShoppingTrip {
///           toys: [DanglerPoleToys] @deprecated(reason: "Cats are too spoiled")
///           """Dry or wet food?"""
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
    description: Option<String>,
    // The vector of interfaces that an object implements.
    interfaces: Vec<String>,
    // The vector of fields query‐able on this type.
    fields: Vec<Field>,
}

impl ObjectDef {
    /// Create a new instance of ObjectDef with a name.
    pub fn new(name: String) -> Self {
        Self {
            name,
            description: None,
            interfaces: Vec::new(),
            fields: Vec::new(),
        }
    }

    /// Set the ObjectDef's description field.
    pub fn description(&mut self, description: Option<String>) {
        self.description = description
    }

    /// Set the interfaces ObjectDef implements.
    pub fn interface(&mut self, interface: String) {
        self.interfaces.push(interface)
    }

    /// Push a Field to ObjectDef's fields vector.
    pub fn field(&mut self, field: Field) {
        self.fields.push(field)
    }
}

impl fmt::Display for ObjectDef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(description) = &self.description {
            // We are determing on whether to have description formatted as
            // a multiline comment based on whether or not it already includes a
            // \n.
            match description.contains('\n') {
                true => writeln!(f, "\"\"\"\n{}\n\"\"\"", description)?,
                false => writeln!(f, "\"\"\"{}\"\"\"", description)?,
            }
        }

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
        let ty_1 = Type_::NamedType {
            name: "DanglerPoleToys".to_string(),
        };

        let ty_2 = Type_::List { ty: Box::new(ty_1) };
        let field = Field::new("toys".to_string(), ty_2);

        let mut object_def = ObjectDef::new("PetStoreTrip".to_string());
        object_def.field(field);
        object_def.description(Some("What to get at Fressnapf?".to_string()));

        assert_eq!(
            object_def.to_string(),
            indoc! { r#"
                """What to get at Fressnapf?"""
                type PetStoreTrip {
                  toys: [DanglerPoleToys]
                }
            "#}
        );
    }

    #[test]
    fn it_encodes_object_with_interface() {
        let ty_1 = Type_::NamedType {
            name: "DanglerPoleToys".to_string(),
        };

        let ty_2 = Type_::List { ty: Box::new(ty_1) };
        let mut field = Field::new("toys".to_string(), ty_2);
        field.deprecated(Some("Cats are too spoiled".to_string()));
        let ty_3 = Type_::NamedType {
            name: "FoodType".to_string(),
        };
        let mut field_2 = Field::new("food".to_string(), ty_3);
        field_2.description(Some("Dry or wet food?".to_string()));

        let ty_4 = Type_::NamedType {
            name: "Boolean".to_string(),
        };
        let field_3 = Field::new("catGrass".to_string(), ty_4);

        let mut object_def = ObjectDef::new("PetStoreTrip".to_string());
        object_def.field(field);
        object_def.field(field_2);
        object_def.field(field_3);
        object_def.interface("ShoppingTrip".to_string());

        assert_eq!(
            object_def.to_string(),
            indoc! { r#"
                type PetStoreTrip implements ShoppingTrip {
                  toys: [DanglerPoleToys] @deprecated(reason: "Cats are too spoiled")
                  """Dry or wet food?"""
                  food: FoodType
                  catGrass: Boolean
                }
            "#}
        );
    }
}
