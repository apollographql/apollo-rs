use std::fmt;

use crate::{Field, StringValue};

/// InterfaceDefs are an abstract type where there are common fields declared.
///
/// Any type that implements an interface must define all the fields with names
/// and types exactly matching. The implementations of this interface are
/// explicitly listed out in possibleTypes.
///
/// *InterfaceDefTypeDefinition*:
///     Description? **interface** Name ImplementsInterfaceDefs? Directives?
///
/// Detailed documentation can be found in [GraphQL spec](https://spec.graphql.org/October2021/#sec-InterfaceDef).
///
/// ### Example
/// ```rust
/// use apollo_encoder::{Type_, Field, InterfaceDef};
/// use indoc::indoc;
///
/// let ty_1 = Type_::NamedType {
///     name: "String".to_string(),
/// };
///
/// let ty_2 = Type_::NamedType {
///     name: "String".to_string(),
/// };
///
/// let ty_3 = Type_::NonNull { ty: Box::new(ty_2) };
/// let ty_4 = Type_::List { ty: Box::new(ty_3) };
/// let ty_5 = Type_::NonNull { ty: Box::new(ty_4) };
///
/// let ty_6 = Type_::NamedType {
///     name: "Boolean".to_string(),
/// };
///
/// let mut field_1 = Field::new("main", ty_1);
/// field_1.description("Cat's main dish of a meal.");
///
/// let mut field_2 = Field::new("snack", ty_5);
/// field_2.description("Cat's post meal snack.");
///
/// let mut field_3 = Field::new("pats", ty_6);
/// field_3.description("Does cat get a pat after meal?");
///
/// // a schema definition
/// let mut interface = InterfaceDef::new("Meal");
/// interface.description("Meal interface for various\nmeals during the day.");
/// interface.field(field_1);
/// interface.field(field_2);
/// interface.field(field_3);
///
/// assert_eq!(
///     interface.to_string(),
///     indoc! { r#"
///     """
///     Meal interface for various
///     meals during the day.
///     """
///     interface Meal {
///       "Cat's main dish of a meal."
///       main: String
///       "Cat's post meal snack."
///       snack: [String!]!
///       "Does cat get a pat after meal?"
///       pats: Boolean
///     }
///     "# }
/// );
/// ```
#[derive(Debug, Clone)]
pub struct InterfaceDef {
    // Name must return a String.
    name: String,
    // Description may return a String or null.
    description: StringValue,
    // The vector of interfaces that this interface implements.
    interfaces: Vec<String>,
    // The vector of fields required by this interface.
    fields: Vec<Field>,
}

impl InterfaceDef {
    /// Create a new instance of InterfaceDef.
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_owned(),
            description: StringValue::Top { source: None },
            fields: Vec::new(),
            interfaces: Vec::new(),
        }
    }

    /// Set the schema def's description.
    pub fn description(&mut self, description: &str) {
        self.description = StringValue::Top {
            source: Some(description.to_owned()),
        };
    }

    /// Set the interfaces ObjectDef implements.
    pub fn interface(&mut self, interface: &str) {
        self.interfaces.push(interface.to_owned())
    }

    /// Push a Field to schema def's fields vector.
    pub fn field(&mut self, field: Field) {
        self.fields.push(field)
    }
}

impl fmt::Display for InterfaceDef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.description)?;

        write!(f, "interface {}", &self.name)?;
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
    use crate::Type_;
    use indoc::indoc;
    use pretty_assertions::assert_eq;

    #[test]
    fn it_encodes_interfaces() {
        let ty_1 = Type_::NamedType {
            name: "String".to_string(),
        };

        let ty_2 = Type_::NamedType {
            name: "String".to_string(),
        };

        let ty_3 = Type_::NonNull { ty: Box::new(ty_2) };
        let ty_4 = Type_::List { ty: Box::new(ty_3) };
        let ty_5 = Type_::NonNull { ty: Box::new(ty_4) };

        let ty_6 = Type_::NamedType {
            name: "Boolean".to_string(),
        };

        let mut field_1 = Field::new("main", ty_1);
        field_1.description("Cat's main dish of a meal.");

        let mut field_2 = Field::new("snack", ty_5);
        field_2.description("Cat's post meal snack.");

        let mut field_3 = Field::new("pats", ty_6);
        field_3.description("Does cat get a pat\nafter meal?");

        // a schema definition
        let mut interface = InterfaceDef::new("Meal");
        interface.description("Meal interface for various\nmeals during the day.");
        interface.field(field_1);
        interface.field(field_2);
        interface.field(field_3);

        assert_eq!(
            interface.to_string(),
            indoc! { r#"
            """
            Meal interface for various
            meals during the day.
            """
            interface Meal {
              "Cat's main dish of a meal."
              main: String
              "Cat's post meal snack."
              snack: [String!]!
              """
              Does cat get a pat
              after meal?
              """
              pats: Boolean
            }
            "# }
        );
    }
}
