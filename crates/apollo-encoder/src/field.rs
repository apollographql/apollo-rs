use std::fmt;

use crate::{
    Argument, ArgumentsDefinition, Directive, InputValueDefinition, SelectionSet, StringValue,
    Type_,
};
/// The FieldDefinition type represents each field definition in an Object
/// definition or Interface type definition.
///
/// *FieldDefinition*:
///     Description? Name ArgumentsDefinition? **:** Type Directives?
///
/// Detailed documentation can be found in [GraphQL spec](https://spec.graphql.org/October2021/#FieldDefinition).
///
/// ### Example
/// ```rust
/// use apollo_encoder::{Type_, FieldDefinition, InputValueDefinition};
///
/// let ty_1 = Type_::NamedType {
///     name: "CatBreed".to_string(),
/// };
///
/// let mut field = FieldDefinition::new("cat".to_string(), ty_1);
///
/// let value_1 = Type_::NamedType {
///     name: "CatBreed".to_string(),
/// };
///
/// let arg = InputValueDefinition::new("breed".to_string(), value_1);
///
/// field.arg(arg);
///
/// assert_eq!(
///     field.to_string(),
///     r#"  cat(breed: CatBreed): CatBreed"#
/// );
/// ```
#[derive(Debug, PartialEq, Clone)]
pub struct FieldDefinition {
    // Name must return a String.
    name: String,
    // Description may return a String.
    description: Option<StringValue>,
    // Args returns a List of __InputValue representing the arguments this field accepts.
    args: ArgumentsDefinition,
    // Type must return a __Type that represents the type of value returned by this field.
    type_: Type_,
    /// Contains all directives.
    directives: Vec<Directive>,
}

impl FieldDefinition {
    /// Create a new instance of Field.
    pub fn new(name: String, type_: Type_) -> Self {
        Self {
            description: None,
            name,
            type_,
            args: ArgumentsDefinition::new(),
            directives: Vec::new(),
        }
    }

    /// Set the Field's description.
    pub fn description(&mut self, description: String) {
        self.description = Some(StringValue::Field {
            source: description,
        });
    }

    /// Set the Field's arguments.
    pub fn arg(&mut self, arg: InputValueDefinition) {
        self.args.input_value(arg);
    }

    /// Add a directive.
    pub fn directive(&mut self, directive: Directive) {
        self.directives.push(directive)
    }
}

impl fmt::Display for FieldDefinition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(description) = &self.description {
            write!(f, "{}", description)?;
        }
        write!(f, "  {}", self.name)?;

        if !self.args.input_values.is_empty() {
            write!(f, "{}", self.args)?;
        }

        write!(f, ": {}", self.type_)?;

        for directive in &self.directives {
            write!(f, " {}", directive)?;
        }

        Ok(())
    }
}

/// The __Field type represents each field in an Object or Interface type.
///
/// *Field*:
///     Alias? Name Arguments? Directives? SelectionSet?
///
/// Detailed documentation can be found in [GraphQL spec](https://spec.graphql.org/October2021/#sec-Language.Fields).
///
/// ### Example
/// ```rust
/// use apollo_encoder::Field;
///
/// let mut field = Field::new("myField".to_string());
/// field.alias(Some("myAlias".to_string()));
///
/// assert_eq!(field.to_string(), r#"myAlias: myField"#);
/// ```
#[derive(Debug, PartialEq, Clone)]
pub struct Field {
    // Alias must be a String.
    alias: Option<String>,
    // Name must return a String.
    name: String,
    // Args returns a List of Argument representing the arguments this field accepts.
    args: Vec<Argument>,
    /// Contains all directives.
    directives: Vec<Directive>,
    selection_set: Option<SelectionSet>,
}

impl Field {
    /// Create an instance of Field
    pub fn new(name: String) -> Self {
        Self {
            name,
            selection_set: None,
            alias: None,
            args: Vec::new(),
            directives: Vec::new(),
        }
    }

    /// Set an alias to a field name
    pub fn alias(&mut self, alias: Option<String>) {
        self.alias = alias;
    }

    /// Add a directive to a field
    pub fn directive(&mut self, directive: Directive) {
        self.directives.push(directive);
    }

    /// Add an argument to a field
    pub fn argument(&mut self, argument: Argument) {
        self.args.push(argument);
    }

    /// Set a selection set to a field
    pub fn selection_set(&mut self, selection_set: Option<SelectionSet>) {
        self.selection_set = selection_set;
    }

    /// Should be used everywhere in this crate instead of the Display implementation
    /// Display implementation is only useful as a public api
    pub(crate) fn format_with_indent(&self, indent_level: usize) -> String {
        let mut text = match &self.alias {
            Some(alias) => format!("{alias}: {}", self.name),
            None => String::from(&self.name),
        };

        if !self.args.is_empty() {
            for (i, arg) in self.args.iter().enumerate() {
                match i {
                    0 => {
                        text.push_str(&format!("({arg}"));
                    }
                    _ => text.push_str(&format!(", {arg}")),
                }
            }
            text.push(')');
        }

        for directive in &self.directives {
            text.push_str(&format!(" {directive}"));
        }
        if let Some(sel_set) = &self.selection_set {
            text.push_str(&format!(" {}", sel_set.format_with_indent(indent_level)));
        }

        text
    }
}

impl fmt::Display for Field {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let indent_level = 0;
        write!(f, "{}", self.format_with_indent(indent_level))
    }
}

#[cfg(test)]
mod tests {
    use crate::{Argument, Value};

    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn it_encodes_simple_fields() {
        let mut field = Field::new("myField".to_string());
        field.alias(Some("myAlias".to_string()));

        assert_eq!(field.to_string(), r#"myAlias: myField"#);
    }

    #[test]
    fn it_encodes_simple_fields_def() {
        let ty_1 = Type_::NamedType {
            name: "SpaceProgram".to_string(),
        };

        let ty_2 = Type_::List { ty: Box::new(ty_1) };
        let ty_3 = Type_::NonNull { ty: Box::new(ty_2) };
        let field = FieldDefinition::new("spaceCat".to_string(), ty_3);

        assert_eq!(field.to_string(), r#"  spaceCat: [SpaceProgram]!"#);
    }

    #[test]
    fn it_encodes_fields_with_directive() {
        let ty_1 = Type_::NamedType {
            name: "SpaceProgram".to_string(),
        };

        let ty_2 = Type_::List { ty: Box::new(ty_1) };
        let mut field = FieldDefinition::new("cat".to_string(), ty_2);
        let mut directive = Directive::new(String::from("testDirective"));
        directive.arg(Argument::new(String::from("first"), Value::Int(1)));
        field.description("Very good cats".to_string());
        field.directive(directive);

        assert_eq!(
            field.to_string(),
            r#"  "Very good cats"
  cat: [SpaceProgram] @testDirective(first: 1)"#
        );
    }

    #[test]
    fn it_encodes_fields_with_description() {
        let ty_1 = Type_::NamedType {
            name: "SpaceProgram".to_string(),
        };

        let ty_2 = Type_::NonNull { ty: Box::new(ty_1) };
        let ty_3 = Type_::List { ty: Box::new(ty_2) };
        let ty_4 = Type_::NonNull { ty: Box::new(ty_3) };
        let mut field = FieldDefinition::new("spaceCat".to_string(), ty_4);
        field.description("Very good space cats".to_string());

        assert_eq!(
            field.to_string(),
            r#"  "Very good space cats"
  spaceCat: [SpaceProgram!]!"#
        );
    }

    #[test]
    fn it_encodes_fields_with_value_arguments() {
        let ty_1 = Type_::NamedType {
            name: "SpaceProgram".to_string(),
        };

        let ty_2 = Type_::NonNull { ty: Box::new(ty_1) };
        let ty_3 = Type_::List { ty: Box::new(ty_2) };
        let ty_4 = Type_::NonNull { ty: Box::new(ty_3) };
        let mut field_definition = FieldDefinition::new("spaceCat".to_string(), ty_4);
        field_definition.description("Very good space cats".to_string());

        let value_1 = Type_::NamedType {
            name: "SpaceProgram".to_string(),
        };

        let value_2 = Type_::List {
            ty: Box::new(value_1),
        };
        let mut arg = InputValueDefinition::new("cat".to_string(), value_2);
        let mut deprecated_directive = Directive::new(String::from("deprecated"));
        deprecated_directive.arg(Argument::new(
            String::from("reason"),
            Value::String(String::from("Cats are no longer sent to space.")),
        ));
        arg.directive(deprecated_directive);
        field_definition.arg(arg);

        assert_eq!(
            field_definition.to_string(),
            r#"  "Very good space cats"
  spaceCat(cat: [SpaceProgram] @deprecated(reason: "Cats are no longer sent to space.")): [SpaceProgram!]!"#
        );
    }

    #[test]
    fn it_encodes_fields_with_argument_descriptions() {
        let ty = Type_::NamedType {
            name: "Cat".to_string(),
        };

        let mut field_definition = FieldDefinition::new("spaceCat".to_string(), ty);

        let value = Type_::NamedType {
            name: "Treat".to_string(),
        };

        let mut arg = InputValueDefinition::new("treat".to_string(), value);
        arg.description("The type of treats given in space".to_string());
        field_definition.arg(arg);

        let value = Type_::NamedType {
            name: "Int".to_string(),
        };

        let mut arg = InputValueDefinition::new("age".to_string(), value);
        arg.description("Optimal age of a \"space\" cat".to_string());
        field_definition.arg(arg);

        assert_eq!(
            field_definition.to_string(),
            r#"  spaceCat(
    "The type of treats given in space"
    treat: Treat,
    """
    Optimal age of a "space" cat
    """
    age: Int
  ): Cat"#
        );
    }
}
