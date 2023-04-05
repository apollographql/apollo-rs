use std::fmt;

use crate::{Directive, FieldDefinition, StringValue};
/// ObjectDefinition types represent concrete instantiations of sets of fields.
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
/// use apollo_encoder::{Type_, FieldDefinition, ObjectDefinition};
/// use indoc::indoc;
///
/// let ty_1 = Type_::NamedType {
///     name: "DanglerPoleToys".to_string(),
/// };
///
/// let ty_2 = Type_::List { ty: Box::new(ty_1) };
/// let mut field = FieldDefinition::new("toys".to_string(), ty_2);
/// let ty_3 = Type_::NamedType {
///     name: "FoodType".to_string(),
/// };
/// let mut field_2 = FieldDefinition::new("food".to_string(), ty_3);
/// field_2.description("Dry or wet food?".to_string());
///
/// let ty_4 = Type_::NamedType {
///     name: "Boolean".to_string(),
/// };
/// let field_3 = FieldDefinition::new("catGrass".to_string(), ty_4);
///
/// let mut object_def = ObjectDefinition::new("PetStoreTrip".to_string());
/// object_def.field(field);
/// object_def.field(field_2);
/// object_def.field(field_3);
/// object_def.interface("ShoppingTrip".to_string());
///
/// assert_eq!(
///     object_def.to_string(),
///     indoc! { r#"
///         type PetStoreTrip implements ShoppingTrip {
///           toys: [DanglerPoleToys]
///           "Dry or wet food?"
///           food: FoodType
///           catGrass: Boolean
///         }
///     "#}
/// );
/// ```
#[derive(Debug, Clone)]
pub struct ObjectDefinition {
    // Name must return a String.
    name: String,
    // Description may return a String or null.
    description: Option<StringValue>,
    // The vector of interfaces that an object implements.
    interfaces: Vec<String>,
    // The vector of fields query‐able on this type.
    fields: Vec<FieldDefinition>,
    /// The vector of directives for this object
    directives: Vec<Directive>,
    extend: bool,
}

impl ObjectDefinition {
    /// Create a new instance of ObjectDef with a name.
    pub fn new(name: String) -> Self {
        Self {
            name,
            description: None,
            interfaces: Vec::new(),
            fields: Vec::new(),
            directives: Vec::new(),
            extend: false,
        }
    }

    /// Set the ObjectDef's description field.
    pub fn description(&mut self, description: String) {
        self.description = Some(StringValue::Top {
            source: description,
        });
    }

    /// Add a directive on ObjectDef.
    pub fn directive(&mut self, directive: Directive) {
        self.directives.push(directive)
    }

    /// Set the object type as an extension
    pub fn extend(&mut self) {
        self.extend = true;
    }

    /// Push a Field to ObjectDef's fields vector.
    pub fn field(&mut self, field: FieldDefinition) {
        self.fields.push(field)
    }

    /// Add an interface ObjectDef implements.
    pub fn interface(&mut self, interface: String) {
        self.interfaces.push(interface)
    }
}

impl fmt::Display for ObjectDefinition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.extend {
            write!(f, "extend ")?;
        // No description when it's a extension
        } else if let Some(description) = &self.description {
            write!(f, "{description}")?;
        }

        write!(f, "type {}", &self.name)?;
        for (i, interface) in self.interfaces.iter().enumerate() {
            match i {
                0 => write!(f, " implements {interface}")?,
                _ => write!(f, " & {interface}")?,
            }
        }
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
    use crate::{Argument, FieldDefinition, InputValueDefinition, Type_, Value};
    use indoc::indoc;
    use pretty_assertions::assert_eq;

    #[test]
    fn it_encodes_object_with_description() {
        let ty_1 = Type_::NamedType {
            name: "DanglerPoleToys".to_string(),
        };

        let ty_2 = Type_::List { ty: Box::new(ty_1) };
        let field = FieldDefinition::new("toys".to_string(), ty_2);

        let mut object_def = ObjectDefinition::new("PetStoreTrip".to_string());
        object_def.field(field);
        object_def.description("What to get at Fressnapf?".to_string());

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
        let ty_1 = Type_::NamedType {
            name: "DanglerPoleToys".to_string(),
        };

        let mut field = FieldDefinition::new("toys".to_string(), ty_1);
        let mut deprecated_directive = Directive::new(String::from("deprecated"));
        deprecated_directive.arg(Argument::new(
            String::from("reason"),
            Value::String(String::from(
                "\"DanglerPoleToys\" are no longer interesting",
            )),
        ));
        field.directive(deprecated_directive);

        let mut object_def = ObjectDefinition::new("PetStoreTrip".to_string());
        object_def.field(field);
        object_def.description("What to get at Fressnapf?".to_string());

        assert_eq!(
            object_def.to_string(),
            indoc! { r#"
                "What to get at Fressnapf?"
                type PetStoreTrip {
                  toys: DanglerPoleToys @deprecated(reason: """"DanglerPoleToys" are no longer interesting""")
                }
            "#}
        );
    }

    #[test]
    fn it_encodes_object_with_extend() {
        let ty_1 = Type_::NamedType {
            name: "DanglerPoleToys".to_string(),
        };

        let mut field = FieldDefinition::new("toys".to_string(), ty_1);
        let mut deprecated_directive = Directive::new(String::from("deprecated"));
        deprecated_directive.arg(Argument::new(
            String::from("reason"),
            Value::String(String::from("DanglerPoleToys are no longer interesting")),
        ));
        field.directive(deprecated_directive);

        let mut object_def = ObjectDefinition::new("PetStoreTrip".to_string());
        object_def.field(field);
        object_def.description("What to get at Fressnapf?".to_string());
        object_def.extend();

        assert_eq!(
            object_def.to_string(),
            indoc! { r#"
                extend type PetStoreTrip {
                  toys: DanglerPoleToys @deprecated(reason: "DanglerPoleToys are no longer interesting")
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
        let mut field = FieldDefinition::new("toys".to_string(), ty_2);
        let mut deprecated_directive = Directive::new(String::from("deprecated"));
        deprecated_directive.arg(Argument::new(
            String::from("reason"),
            Value::String(String::from("Cats are too spoiled")),
        ));
        field.directive(deprecated_directive);
        let ty_3 = Type_::NamedType {
            name: "FoodType".to_string(),
        };
        let mut field_2 = FieldDefinition::new("food".to_string(), ty_3);
        field_2.description("Dry or wet food?".to_string());

        let ty_4 = Type_::NamedType {
            name: "Boolean".to_string(),
        };
        let field_3 = FieldDefinition::new("catGrass".to_string(), ty_4);

        let mut object_def = ObjectDefinition::new("PetStoreTrip".to_string());
        object_def.field(field);
        object_def.field(field_2);
        object_def.field(field_3);
        object_def.description("Shopping list for cats at the pet store.".to_string());
        object_def.interface("ShoppingTrip".to_string());

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
    fn it_encodes_object_with_interface_and_directives() {
        let ty_1 = Type_::NamedType {
            name: "DanglerPoleToys".to_string(),
        };

        let mut directive = Directive::new(String::from("testDirective"));
        directive.arg(Argument::new(
            String::from("first"),
            Value::String("one".to_string()),
        ));
        let ty_2 = Type_::List { ty: Box::new(ty_1) };
        let mut field = FieldDefinition::new("toys".to_string(), ty_2);
        let mut deprecated_directive = Directive::new(String::from("deprecated"));
        deprecated_directive.arg(Argument::new(
            String::from("reason"),
            Value::String(String::from("Cats are too spoiled")),
        ));
        field.directive(deprecated_directive);
        let ty_3 = Type_::NamedType {
            name: "FoodType".to_string(),
        };
        let mut field_2 = FieldDefinition::new("food".to_string(), ty_3);
        field_2.description("Dry or wet food?".to_string());

        let ty_4 = Type_::NamedType {
            name: "Boolean".to_string(),
        };
        let mut field_3 = FieldDefinition::new("catGrass".to_string(), ty_4);

        let value = Type_::NamedType {
            name: "Boolean".to_string(),
        };

        let mut arg = InputValueDefinition::new("fresh".to_string(), value);
        arg.description("How fresh is this grass".to_string());
        field_3.arg(arg);

        let mut object_def = ObjectDefinition::new("PetStoreTrip".to_string());
        object_def.field(field);
        object_def.field(field_2);
        object_def.field(field_3);
        object_def.description("Shopping list for cats at the pet store.".to_string());
        object_def.interface("ShoppingTrip".to_string());
        object_def.directive(directive);

        assert_eq!(
            object_def.to_string(),
            indoc! { r#"
                "Shopping list for cats at the pet store."
                type PetStoreTrip implements ShoppingTrip @testDirective(first: "one") {
                  toys: [DanglerPoleToys] @deprecated(reason: "Cats are too spoiled")
                  "Dry or wet food?"
                  food: FoodType
                  catGrass(
                    "How fresh is this grass"
                    fresh: Boolean
                  ): Boolean
                }
            "#}
        );
    }

    #[test]
    fn it_encodes_object_with_input_value_definition_fields() {
        let ty_1 = Type_::NamedType {
            name: "DanglerPoleToys".to_string(),
        };

        let ty_2 = Type_::List { ty: Box::new(ty_1) };
        let field = FieldDefinition::new("toys".to_string(), ty_2);

        let ty_4 = Type_::NamedType {
            name: "Boolean".to_string(),
        };
        let mut field_3 = FieldDefinition::new("catGrass".to_string(), ty_4);

        let value = Type_::NamedType {
            name: "Boolean".to_string(),
        };

        let mut arg = InputValueDefinition::new("fresh".to_string(), value);
        arg.description("How fresh is this grass".to_string());
        field_3.arg(arg);

        let value = Type_::NamedType {
            name: "Int".to_string(),
        };

        let mut arg = InputValueDefinition::new("quantity".to_string(), value);
        arg.description("Number of pots\ngrowing at a given time".to_string());
        field_3.arg(arg);

        let mut object_def = ObjectDefinition::new("PetStoreTrip".to_string());
        object_def.field(field);
        object_def.field(field_3);
        object_def.description("Shopping list for cats at the pet store.".to_string());

        assert_eq!(
            object_def.to_string(),
            indoc! { r#"
                "Shopping list for cats at the pet store."
                type PetStoreTrip {
                  toys: [DanglerPoleToys]
                  catGrass(
                    "How fresh is this grass"
                    fresh: Boolean,
                    """
                    Number of pots
                    growing at a given time
                    """
                    quantity: Int
                  ): Boolean
                }
            "#}
        );
    }

    #[test]
    fn it_encodes_object_with_block_string_description() {
        let ty_1 = Type_::NamedType {
            name: "String".to_string(),
        };

        let mut field = FieldDefinition::new("name".to_string(), ty_1);
        field.description("multiline\ndescription".to_string());

        let mut object_def = ObjectDefinition::new("Book".to_string());
        object_def.field(field);
        object_def.description("Book Object\nType".to_string());

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
