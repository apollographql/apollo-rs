use std::fmt;

use crate::{Argument, Directive, InputValueDef, SelectionSet, StringValue, Type_};
/// The __FieldDef type represents each field definition in an Object definition or Interface type definition.
///
/// *FieldDefinition*:
///     Description? Name ArgumentsDefinition? **:** Type Directives?
///
/// Detailed documentation can be found in [GraphQL spec](https://spec.graphql.org/October2021/#FieldDefinition).
///
/// ### Example
/// ```rust
/// use apollo_encoder::{Type_, FieldDef, InputValueDef};
///
/// let ty_1 = Type_::NamedType {
///     name: "CatBreed".to_string(),
/// };
///
/// let mut field = FieldDef::new("cat".to_string(), ty_1);
///
/// let value_1 = Type_::NamedType {
///     name: "CatBreed".to_string(),
/// };
///
/// let arg = InputValueDef::new("breed".to_string(), value_1);
///
/// field.arg(arg);
///
/// assert_eq!(
///     field.to_string(),
///     r#"  cat(breed: CatBreed): CatBreed"#
/// );
/// ```
#[derive(Debug, PartialEq, Clone)]
pub struct FieldDef {
    // Name must return a String.
    name: String,
    // Description may return a String.
    description: StringValue,
    // Args returns a List of __InputValue representing the arguments this field accepts.
    args: Vec<InputValueDef>,
    // Type must return a __Type that represents the type of value returned by this field.
    type_: Type_,
    /// Contains all directives.
    directives: Vec<Directive>,
}

impl FieldDef {
    /// Create a new instance of Field.
    pub fn new(name: String, type_: Type_) -> Self {
        Self {
            description: StringValue::Field { source: None },
            name,
            type_,
            args: Vec::new(),
            directives: Vec::new(),
        }
    }

    /// Set the Field's description.
    pub fn description(&mut self, description: Option<String>) {
        self.description = StringValue::Field {
            source: description,
        };
    }

    /// Set the Field's arguments.
    pub fn arg(&mut self, arg: InputValueDef) {
        self.args.push(arg);
    }

    /// Add a directive.
    pub fn directive(&mut self, directive: Directive) {
        self.directives.push(directive)
    }
}

impl fmt::Display for FieldDef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.description)?;
        write!(f, "  {}", self.name)?;

        if !self.args.is_empty() {
            for (i, arg) in self.args.iter().enumerate() {
                match i {
                    0 => write!(f, "({}", arg)?,
                    _ => write!(f, ", {}", arg)?,
                }
            }
            write!(f, ")")?;
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
        let mut text = String::from(&self.name);

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

        if let Some(alias) = &self.alias {
            write!(f, "{}: ", alias)?;
        }
        write!(f, "{}", self.name)?;

        if !self.args.is_empty() {
            for (i, arg) in self.args.iter().enumerate() {
                match i {
                    0 => write!(f, "({}", arg)?,
                    _ => write!(f, ", {}", arg)?,
                }
            }
            write!(f, ")")?;
        }

        for directive in &self.directives {
            write!(f, " {}", directive)?;
        }
        if let Some(sel_set) = &self.selection_set {
            write!(f, " {}", sel_set.format_with_indent(indent_level))?;
        }

        Ok(())
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
        let field = FieldDef::new("spaceCat".to_string(), ty_3);

        assert_eq!(field.to_string(), r#"  spaceCat: [SpaceProgram]!"#);
    }

    #[test]
    fn it_encodes_fields_with_directive() {
        let ty_1 = Type_::NamedType {
            name: "SpaceProgram".to_string(),
        };

        let ty_2 = Type_::List { ty: Box::new(ty_1) };
        let mut field = FieldDef::new("cat".to_string(), ty_2);
        let mut directive = Directive::new(String::from("testDirective"));
        directive.arg(Argument::new(String::from("first"), Value::Int(1)));
        field.description(Some("Very good cats".to_string()));
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
        let mut field = FieldDef::new("spaceCat".to_string(), ty_4);
        field.description(Some("Very good space cats".to_string()));

        assert_eq!(
            field.to_string(),
            r#"  "Very good space cats"
  spaceCat: [SpaceProgram!]!"#
        );
    }

    #[test]
    fn it_encodes_fields_with_valueuments() {
        let ty_1 = Type_::NamedType {
            name: "SpaceProgram".to_string(),
        };

        let ty_2 = Type_::NonNull { ty: Box::new(ty_1) };
        let ty_3 = Type_::List { ty: Box::new(ty_2) };
        let ty_4 = Type_::NonNull { ty: Box::new(ty_3) };
        let mut field = FieldDef::new("spaceCat".to_string(), ty_4);
        field.description(Some("Very good space cats".to_string()));

        let value_1 = Type_::NamedType {
            name: "SpaceProgram".to_string(),
        };

        let value_2 = Type_::List {
            ty: Box::new(value_1),
        };
        let mut arg = InputValueDef::new("cat".to_string(), value_2);
        let mut deprecated_directive = Directive::new(String::from("deprecated"));
        deprecated_directive.arg(Argument::new(
            String::from("reason"),
            Value::String(String::from("Cats are no longer sent to space.")),
        ));
        arg.directive(deprecated_directive);
        field.arg(arg);

        assert_eq!(
            field.to_string(),
            r#"  "Very good space cats"
  spaceCat(cat: [SpaceProgram] @deprecated(reason: "Cats are no longer sent to space.")): [SpaceProgram!]!"#
        );
    }
}
