use std::fmt;

use crate::{ArgumentsDefinition, InputValueDefinition, StringValue};

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
/// directive.description("Infer field types\nfrom field values.".to_string());
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
    description: Option<StringValue>,
    // Args returns a Vector of __InputValue representing the arguments this
    // directive accepts.
    args: ArgumentsDefinition,
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
            description: None,
            args: ArgumentsDefinition::new(),
            locations: Vec::new(),
            repeatable: false,
        }
    }

    /// Set the Directive's description.
    pub fn description(&mut self, description: String) {
        self.description = Some(StringValue::Top {
            source: description,
        });
    }

    /// Add a location where this Directive can be used.
    pub fn location(&mut self, location: String) {
        self.locations.push(location);
    }

    /// Add an argument to this Directive.
    pub fn arg(&mut self, arg: InputValueDefinition) {
        self.args.input_value(arg);
    }

    /// Set the Directive's repeatable
    pub fn repeatable(&mut self) {
        self.repeatable = true;
    }
}

impl fmt::Display for DirectiveDefinition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(description) = &self.description {
            write!(f, "{}", description)?;
        }
        write!(f, "directive @{}", self.name)?;

        if !self.args.input_values.is_empty() {
            write!(f, "{}", self.args)?;
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
        directive.description("Infer field types from field values.".to_string());
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
        directive.description("Infer field types\nfrom field values.".to_string());
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
        directive.description("Infer field types from field values.".to_string());
        directive.location("OBJECT".to_string());

        let ty_1 = Type_::NamedType {
            name: "SpaceProgram".to_string(),
        };

        let ty_2 = Type_::List { ty: Box::new(ty_1) };
        let arg = InputValueDefinition::new("cat".to_string(), ty_2);
        directive.arg(arg);

        assert_eq!(
            directive.to_string(),
            r#""Infer field types from field values."
directive @infer(cat: [SpaceProgram]) on OBJECT
"#
        );
    }

    #[test]
    fn it_encodes_directives_with_arguments_with_description() {
        let mut directive = DirectiveDefinition::new("infer".to_string());
        directive.description("Infer field types from field values.".to_string());
        directive.location("OBJECT".to_string());

        let ty_1 = Type_::NamedType {
            name: "SpaceProgram".to_string(),
        };

        let ty_2 = Type_::List { ty: Box::new(ty_1) };
        let mut arg = InputValueDefinition::new("cat".to_string(), ty_2);
        arg.description("Space Program for flying cats".to_string());
        directive.arg(arg);

        assert_eq!(
            directive.to_string(),
            r#""Infer field types from field values."
directive @infer(
    "Space Program for flying cats"
    cat: [SpaceProgram]
  ) on OBJECT
"#
        );
    }
}
