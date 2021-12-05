use std::fmt;

use crate::{InputValue, StringValue};

/// The `__Directive` type represents a Directive that a service supports.
///
/// *DirectiveDefinition*:
///     Description? **directive @** Name Arguments Definition? **repeatable**? **on** DirectiveLocations
///
/// Detailed documentation can be found in [GraphQL spec](https://spec.graphql.org/October2021/#sec-Type-System.Directives).
///
/// ### Example
/// ```rust
/// use apollo_encoder::{Directive};
/// use indoc::indoc;
///
/// let mut directive = Directive::new("infer");
/// directive.description("Infer field types\nfrom field values.");
/// directive.location("OBJECT");
/// directive.location("FIELD_DEFINITION");
/// directive.location("INPUT_FIELD_DEFINITION");
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
pub struct Directive {
    // Name must return a String.
    name: String,
    // Description may return a String or null.
    description: StringValue,
    // Args returns a Vector of __InputValue representing the arguments this
    // directive accepts.
    args: Vec<InputValue>,
    // Locations returns a List of __DirectiveLocation representing the valid
    // locations this directive may be placed.
    locations: Vec<String>,
}

impl Directive {
    /// Create a new instance of Directive definition.
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            description: StringValue::Top { source: None },
            args: Vec::new(),
            locations: Vec::new(),
        }
    }

    /// Set the Directive's description.
    pub fn description(&mut self, description: &str) {
        self.description = StringValue::Top {
            source: Some(description.to_string()),
        };
    }

    /// Set the Directive's location.
    pub fn location(&mut self, location: &str) {
        self.locations.push(location.to_string());
    }

    /// Set the Directive's args.
    pub fn arg(&mut self, arg: InputValue) {
        self.args.push(arg);
    }
}

impl fmt::Display for Directive {
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
    // use indoc::indoc;
    use crate::Type_;
    use pretty_assertions::assert_eq;

    #[test]
    fn it_encodes_directives_for_a_single_location() {
        let directive = {
            let mut directive = Directive::new("infer");
            directive.description("Infer field types from field values.");
            directive.location("OBJECT");
            directive
        };

        assert_eq!(
            directive.to_string(),
            r#""Infer field types from field values."
directive @infer on OBJECT
"#
        );
    }

    #[test]
    fn it_encodes_directives_for_multiple_location() {
        let directive = {
            let mut directive = Directive::new("infer");
            directive.description("Infer field types\nfrom field values.");
            directive.location("OBJECT");
            directive.location("FIELD_DEFINITION");
            directive.location("INPUT_FIELD_DEFINITION");
            directive
        };

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
        let directive = {
            let ty = Type_::named_type("SpaceProgram");
            let ty = Type_::list(Box::new(ty));
            let arg = InputValue::new("cat", ty);

            let mut directive = Directive::new("infer");
            directive.description("Infer field types from field values.");
            directive.location("OBJECT");
            directive.arg(arg);
            directive
        };

        assert_eq!(
            directive.to_string(),
            r#""Infer field types from field values."
directive @infer(cat: [SpaceProgram]) on OBJECT
"#
        );
    }
}
