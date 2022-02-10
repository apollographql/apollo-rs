use std::fmt;

use crate::{InputValueDef, StringValue};

/// The `DirectiveDefinition` type represents a Directive definition.
///
/// *DirectiveDefinition*:
///     Description? **directive @** Name Arguments Definition? **repeatable**? **on** DirectiveLocations
///
/// Detailed documentation can be found in [GraphQL spec](https://spec.graphql.org/October2021/#sec-Type-System.Directives).
///
/// ### Example
/// ```rust
/// use apollo_encoder::DirectiveDefinition;
/// use indoc::indoc;
///
/// let mut directive = DirectiveDefinition::new("infer".to_string());
/// directive.description(Some("Infer field types\nfrom field values.".to_string()));
/// directive.location("OBJECT".to_string());
/// directive.location("FIELD_DEFINITION".to_string());
/// directive.location("INPUT_FIELD_DEFINITION".to_string());
///
/// assert_eq!(
///     directive.to_string(),
///     r#""""
/// Infer field types
/// from field values.
/// """
/// directive @infer on OBJECT | FIELD_DEFINITION | INPUT_FIELD_DEFINITION
/// "#
/// );
/// ```
#[derive(Debug)]
pub struct DirectiveDefinition {
    // Name must return a String.
    name: String,
    // Description may return a String or null.
    description: StringValue,
    // Args returns a Vector of __InputValue representing the arguments this
    // directive accepts.
    args: Vec<InputValueDef>,
    // Locations returns a List of __DirectiveLocation representing the valid
    // locations this directive may be placed.
    locations: Vec<String>,
    // If the directive is repeatable
    repeatable: bool,
}

impl DirectiveDefinition {
    /// Create a new instance of Directive definition.
    pub fn new(name: String) -> Self {
        Self {
            name,
            description: StringValue::Top { source: None },
            args: Vec::new(),
            locations: Vec::new(),
            repeatable: false,
        }
    }

    /// Set the Directive's description.
    pub fn description(&mut self, description: Option<String>) {
        self.description = StringValue::Top {
            source: description,
        };
    }

    /// Set the Directive's location.
    pub fn location(&mut self, location: String) {
        self.locations.push(location);
    }

    /// Set the Directive's args.
    pub fn arg(&mut self, arg: InputValueDef) {
        self.args.push(arg);
    }

    /// Set the Directive's repeatable
    pub fn repeatable(&mut self) {
        self.repeatable = true;
    }
}

impl fmt::Display for DirectiveDefinition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.description)?;
        write!(f, "directive @{}", self.name)?;

        if !self.args.is_empty() {
            for (i, arg) in self.args.iter().enumerate() {
                match i {
                    0 => write!(f, "({}", arg)?,
                    _ => write!(f, ", {}", arg)?,
                }
            }
            write!(f, ")")?;
        }

        if self.repeatable {
            write!(f, " repeatable")?;
        }

        for (i, location) in self.locations.iter().enumerate() {
            match i {
                0 => write!(f, " on {}", location)?,
                _ => write!(f, " | {}", location)?,
            }
        }

        // append a new line at the end
        writeln!(f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Type_;
    use pretty_assertions::assert_eq;

    #[test]
    fn it_encodes_directives_for_a_single_location() {
        let mut directive = DirectiveDefinition::new("infer".to_string());
        directive.description(Some("Infer field types from field values.".to_string()));
        directive.location("OBJECT".to_string());

        assert_eq!(
            directive.to_string(),
            r#""Infer field types from field values."
directive @infer on OBJECT
"#
        );
    }

    #[test]
    fn it_encodes_directives_for_multiple_location() {
        let mut directive = DirectiveDefinition::new("infer".to_string());
        directive.description(Some("Infer field types\nfrom field values.".to_string()));
        directive.location("OBJECT".to_string());
        directive.location("FIELD_DEFINITION".to_string());
        directive.location("INPUT_FIELD_DEFINITION".to_string());

        assert_eq!(
            directive.to_string(),
            r#""""
Infer field types
from field values.
"""
directive @infer on OBJECT | FIELD_DEFINITION | INPUT_FIELD_DEFINITION
"#
        );
    }

    #[test]
    fn it_encodes_directives_with_arguments() {
        let mut directive = DirectiveDefinition::new("infer".to_string());
        directive.description(Some("Infer field types from field values.".to_string()));
        directive.location("OBJECT".to_string());

        let ty_1 = Type_::NamedType {
            name: "SpaceProgram".to_string(),
        };

        let ty_2 = Type_::List { ty: Box::new(ty_1) };
        let arg = InputValueDef::new("cat".to_string(), ty_2);
        directive.arg(arg);

        assert_eq!(
            directive.to_string(),
            r#""Infer field types from field values."
directive @infer(cat: [SpaceProgram]) on OBJECT
"#
        );
    }
}
