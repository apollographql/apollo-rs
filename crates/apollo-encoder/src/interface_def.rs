use crate::Field;
use std::fmt::{self, Display};

/// InterfaceDefs are an abstract type where there are common fields declared.
///
/// Any type that implements an interface must define all the fields with names
/// and types exactly matching. The implementations of this interface are
/// explicitly listed out in possibleTypes.
///
/// *InterfaceTypeDefinition*:
///     Description<sub>opt</sub> **interface** Name ImplementsInterface<sub>opt</sub> Directives<sub>\[Const\] opt</sub> FieldsDefinition<sub>opt</sub>
///
/// Detailed documentation can be found in [GraphQL spec](https://spec.graphql.org/draft/#sec-InterfaceDef).
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
/// let mut field_1 = Field::new("main".to_string(), ty_1);
/// field_1.description(Some("Cat's main dish of a meal.".to_string()));
///
/// let mut field_2 = Field::new("snack".to_string(), ty_5);
/// field_2.description(Some("Cat's post meal snack.".to_string()));
///
/// let mut field_3 = Field::new("pats".to_string(), ty_6);
/// field_3.description(Some("Does cat get a pat after meal?".to_string()));
///
/// // a schema definition
/// let mut interface = InterfaceDef::new("Meal".to_string());
/// interface.description(Some(
///     "Meal interface for various meals during the day.".to_string(),
/// ));
/// interface.field(field_1);
/// interface.field(field_2);
/// interface.field(field_3);
///
/// assert_eq!(
///     interface.to_string(),
///     indoc! { r#"
///     """Meal interface for various meals during the day."""
///     interface Meal {
///       """Cat's main dish of a meal."""
///       main: String
///       """Cat's post meal snack."""
///       snack: [String!]!
///       """Does cat get a pat after meal?"""
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
    description: Option<String>,
    // The vector of interfaces that this interface implements.
    interfaces: Vec<String>,
    // The vector of fields required by this interface.
    fields: Vec<Field>,
}

impl InterfaceDef {
    /// Create a new instance of InterfaceDef.
    pub fn new(name: String) -> Self {
        Self {
            name,
            description: None,
            fields: Vec::new(),
            interfaces: Vec::new(),
        }
    }

    /// Set the schema def's description.
    pub fn description(&mut self, description: Option<String>) {
        self.description = description
    }

    /// Set the interfaces ObjectDef implements.
    pub fn interface(&mut self, interface: String) {
        self.interfaces.push(interface)
    }

    /// Push a Field to schema def's fields vector.
    pub fn field(&mut self, field: Field) {
        self.fields.push(field)
    }
}

impl Display for InterfaceDef {
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

        let mut field_1 = Field::new("main".to_string(), ty_1);
        field_1.description(Some("Cat's main dish of a meal.".to_string()));

        let mut field_2 = Field::new("snack".to_string(), ty_5);
        field_2.description(Some("Cat's post meal snack.".to_string()));

        let mut field_3 = Field::new("pats".to_string(), ty_6);
        field_3.description(Some("Does cat get a pat after meal?".to_string()));

        // a schema definition
        let mut interface = InterfaceDef::new("Meal".to_string());
        interface.description(Some(
            "Meal interface for various meals during the day.".to_string(),
        ));
        interface.field(field_1);
        interface.field(field_2);
        interface.field(field_3);

        assert_eq!(
            interface.to_string(),
            indoc! { r#"
            """Meal interface for various meals during the day."""
            interface Meal {
              """Cat's main dish of a meal."""
              main: String
              """Cat's post meal snack."""
              snack: [String!]!
              """Does cat get a pat after meal?"""
              pats: Boolean
            }
            "# }
        );
    }
}
