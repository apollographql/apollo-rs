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

/// ### Example
/// ```rust
/// use apollo_encoder::{Type_, FieldBuilder, InterfaceDefBuilder};
/// use indoc::indoc;
///
/// let field_1 = {
///     let ty = Type_::named_type("String");
///
///     FieldBuilder::new("main", ty)
///         .description("Cat's main dish of a meal.")
///         .build()
/// };
///
/// let field_2 = {
///     let ty = Type_::named_type("String");
///     let ty = Type_::non_null(Box::new(ty));
///     let ty = Type_::list(Box::new(ty));
///     let ty = Type_::non_null(Box::new(ty));
///
///     FieldBuilder::new("snack", ty)
///         .description("Cat's post meal snack.")
///         .build()
/// };
///
/// let field_3 = {
///     let ty = Type_::named_type("Boolean");
///
///     FieldBuilder::new("pats", ty)
///         .description("Does cat get a pat after meal?")
///         .build()
/// };
///
/// // a schema definition
/// let interface = InterfaceDefBuilder::new("Meal")
///     .description("Meal interface for various\nmeals during the day.")
///     .field(field_1)
///     .field(field_2)
///     .field(field_3)
///     .build();
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
pub struct InterfaceDefBuilder {
    // Name must return a String.
    name: String,
    // Description may return a String or null.
    description: Option<String>,
    // The vector of interfaces that this interface implements.
    interfaces: Vec<String>,
    // The vector of fields required by this interface.
    fields: Vec<Field>,
}

impl InterfaceDefBuilder {
    /// Create a new instance of InterfaceDefBuilder.
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            description: None,
            fields: Vec::new(),
            interfaces: Vec::new(),
        }
    }

    /// Set the schema def's description.
    pub fn description(mut self, description: &str) -> Self {
        self.description = Some(description.to_string());
        self
    }

    /// Set the interfaces ObjectDef implements.
    pub fn interface(mut self, interface: &str) -> Self {
        self.interfaces.push(interface.to_string());
        self
    }

    /// Push a Field to schema def's fields vector.
    pub fn field(mut self, field: Field) -> Self {
        self.fields.push(field);
        self
    }

    /// Create a new instance of InterfaceDef.
    pub fn build(self) -> InterfaceDef {
        InterfaceDef {
            name: self.name,
            description: StringValue::Top {
                source: self.description,
            },
            fields: self.fields,
            interfaces: self.interfaces,
        }
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
    use crate::{FieldBuilder, InterfaceDefBuilder, Type_};
    use indoc::indoc;
    use pretty_assertions::assert_eq;

    #[test]
    fn it_encodes_interfaces() {
        let field_1 = {
            let ty = Type_::named_type("String");

            FieldBuilder::new("main", ty)
                .description("Cat's main dish of a meal.")
                .build()
        };

        let field_2 = {
            let ty = Type_::named_type("String");
            let ty = Type_::non_null(Box::new(ty));
            let ty = Type_::list(Box::new(ty));
            let ty = Type_::non_null(Box::new(ty));

            FieldBuilder::new("snack", ty)
                .description("Cat's post meal snack.")
                .build()
        };

        let field_3 = {
            let ty = Type_::named_type("Boolean");

            FieldBuilder::new("pats", ty)
                .description("Does cat get a pat\nafter meal?")
                .build()
        };

        // a schema definition
        let interface = InterfaceDefBuilder::new("Meal")
            .description("Meal interface for various\nmeals during the day.")
            .field(field_1)
            .field(field_2)
            .field(field_3)
            .build();

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
