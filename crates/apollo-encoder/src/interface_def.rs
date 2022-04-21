use std::fmt;

use crate::{Directive, FieldDefinition, StringValue};

/// InterfaceDefinition is an abstract type where there are common fields declared.
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
/// use apollo_encoder::{Type_, FieldDefinition, InterfaceDefinition};
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
/// let mut field_1 = FieldDefinition::new("main".to_string(), ty_1);
/// field_1.description("Cat's main dish of a meal.".to_string());
///
/// let mut field_2 = FieldDefinition::new("snack".to_string(), ty_5);
/// field_2.description("Cat's post meal snack.".to_string());
///
/// let mut field_3 = FieldDefinition::new("pats".to_string(), ty_6);
/// field_3.description("Does cat get a pat after meal?".to_string());
///
/// // a schema definition
/// let mut interface = InterfaceDefinition::new("Meal".to_string());
/// interface.description(
///     "Meal interface for various\nmeals during the day.".to_string(),
/// );
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
pub struct InterfaceDefinition {
    // Name must return a String.
    name: String,
    // Description may return a String or null.
    description: Option<StringValue>,
    // The vector of interfaces that this interface implements.
    interfaces: Vec<String>,
    // The vector of fields required by this interface.
    fields: Vec<FieldDefinition>,
    /// Contains all directives.
    directives: Vec<Directive>,
    extend: bool,
}

impl InterfaceDefinition {
    /// Create a new instance of InterfaceDef.
    pub fn new(name: String) -> Self {
        Self {
            name,
            description: None,
            fields: Vec::new(),
            interfaces: Vec::new(),
            directives: Vec::new(),
            extend: false,
        }
    }

    /// Set the schema def's description.
    pub fn description(&mut self, description: String) {
        self.description = Some(StringValue::Top {
            source: description,
        });
    }

    /// Set the interfaces ObjectDef implements.
    pub fn interface(&mut self, interface: String) {
        self.interfaces.push(interface)
    }

    /// Set the interface as an extension
    pub fn extend(&mut self) {
        self.extend = true;
    }

    /// Push a Field to schema def's fields vector.
    pub fn field(&mut self, field: FieldDefinition) {
        self.fields.push(field)
    }

    /// Add a directive.
    pub fn directive(&mut self, directive: Directive) {
        self.directives.push(directive)
    }
}

impl fmt::Display for InterfaceDefinition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.extend {
            write!(f, "extend ")?;
        } else {
            // No description when it's a extension
            if let Some(description) = &self.description {
                write!(f, "{}", description)?;
            }
        }

        write!(f, "interface {}", &self.name)?;
        for (i, interface) in self.interfaces.iter().enumerate() {
            match i {
                0 => write!(f, " implements {}", interface)?,
                _ => write!(f, "& {}", interface)?,
            }
        }
        for directive in &self.directives {
            write!(f, " {}", directive)?;
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
    use crate::{Argument, Type_, Value};
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

        let mut field_1 = FieldDefinition::new("main".to_string(), ty_1);
        field_1.description("Cat's main dish of a meal.".to_string());

        let mut field_2 = FieldDefinition::new("snack".to_string(), ty_5);
        field_2.description("Cat's post meal snack.".to_string());

        let mut field_3 = FieldDefinition::new("pats".to_string(), ty_6);
        field_3.description("Does cat get a pat\nafter meal?".to_string());

        let mut directive = Directive::new(String::from("testDirective"));
        directive.arg(Argument::new(
            String::from("first"),
            Value::String("one".to_string()),
        ));

        // a schema definition
        let mut interface = InterfaceDefinition::new("Meal".to_string());
        interface.description("Meal interface for various\nmeals during the day.".to_string());
        interface.field(field_1);
        interface.field(field_2);
        interface.field(field_3);
        interface.directive(directive);

        assert_eq!(
            interface.to_string(),
            indoc! { r#"
            """
            Meal interface for various
            meals during the day.
            """
            interface Meal @testDirective(first: "one") {
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

    #[test]
    fn it_encodes_interface_extension() {
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

        let mut field_1 = FieldDefinition::new("main".to_string(), ty_1);
        field_1.description("Cat's main dish of a meal.".to_string());

        let mut field_2 = FieldDefinition::new("snack".to_string(), ty_5);
        field_2.description("Cat's post meal snack.".to_string());

        let mut field_3 = FieldDefinition::new("pats".to_string(), ty_6);
        field_3.description("Does cat get a pat\nafter meal?".to_string());

        let mut directive = Directive::new(String::from("testDirective"));
        directive.arg(Argument::new(
            String::from("first"),
            Value::String("one".to_string()),
        ));

        // a schema definition
        let mut interface = InterfaceDefinition::new("Meal".to_string());
        interface.description("Meal interface for various\nmeals during the day.".to_string());
        interface.field(field_1);
        interface.field(field_2);
        interface.field(field_3);
        interface.directive(directive);
        interface.extend();

        assert_eq!(
            interface.to_string(),
            indoc! { r#"
            extend interface Meal @testDirective(first: "one") {
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
