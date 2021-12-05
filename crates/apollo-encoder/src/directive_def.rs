use std::fmt;

use crate::{InputValue, StringValue};

/// The `__Directive` type represents a Directive that a service supports.
///
/// *DirectiveDefinition*:
///     Description? **directive @** Name Arguments Definition? **repeatable**? **on** DirectiveLocations
///
/// Detailed documentation can be found in [GraphQL spec](https://spec.graphql.org/October2021/#sec-Type-System.Directives).
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

/// ### Example
/// ```rust
/// use apollo_encoder::{DirectiveBuilder};
/// use indoc::indoc;
///
/// let directive = DirectiveBuilder::new("infer")
///     .description("Infer field types\nfrom field values.")
///     .location("OBJECT")
///     .location("FIELD_DEFINITION")
///     .location("INPUT_FIELD_DEFINITION")
///     .build();
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
pub struct DirectiveBuilder {
    // Name must return a String.
    name: String,
    // Description may return a String or null.
    description: Option<String>,
    // Args returns a Vector of __InputValue representing the arguments this
    // directive accepts.
    args: Vec<InputValue>,
    // Locations returns a List of __DirectiveLocation representing the valid
    // locations this directive may be placed.
    locations: Vec<String>,
}

impl DirectiveBuilder {
    /// Create a new instance of DirectiveBuilder.
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            description: None,
            args: Vec::new(),
            locations: Vec::new(),
        }
    }

    /// Set the Directive's description.
    pub fn description(mut self, description: &str) -> Self {
        self.description = Some(description.to_string());
        self
    }

    /// Set the Directive's location.
    pub fn location(mut self, location: &str) -> Self {
        self.locations.push(location.to_string());
        self
    }

    /// Set the Directive's args.
    pub fn arg(mut self, arg: InputValue) -> Self {
        self.args.push(arg);
        self
    }

    /// Create a new instance of Directive.
    pub fn build(self) -> Directive {
        Directive {
            name: self.name,
            description: StringValue::Top {
                source: self.description,
            },
            args: self.args,
            locations: self.locations,
        }
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
    use crate::{DirectiveBuilder, InputValueBuilder, Type_};
    use pretty_assertions::assert_eq;

    #[test]
    fn it_encodes_directives_for_a_single_location() {
        let directive = DirectiveBuilder::new("infer")
            .description("Infer field types from field values.")
            .location("OBJECT")
            .build();

        assert_eq!(
            directive.to_string(),
            r#""Infer field types from field values."
directive @infer on OBJECT
"#
        );
    }

    #[test]
    fn it_encodes_directives_for_multiple_location() {
        let directive = DirectiveBuilder::new("infer")
            .description("Infer field types\nfrom field values.")
            .location("OBJECT")
            .location("FIELD_DEFINITION")
            .location("INPUT_FIELD_DEFINITION")
            .build();

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
        let arg = {
            let ty = Type_::named_type("SpaceProgram");
            let ty = Type_::list(Box::new(ty));

            InputValueBuilder::new("cat", ty).build()
        };

        let directive = DirectiveBuilder::new("infer")
            .description("Infer field types from field values.")
            .location("OBJECT")
            .arg(arg)
            .build();

        assert_eq!(
            directive.to_string(),
            r#""Infer field types from field values."
directive @infer(cat: [SpaceProgram]) on OBJECT
"#
        );
    }
}
