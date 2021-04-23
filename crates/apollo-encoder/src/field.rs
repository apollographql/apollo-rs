use crate::{InputValue, Type_};
use std::fmt::{self, Display};
/// The __Field type represents each field in an Object or Interface type.
///
/// *FieldDefinition*:
///     Description<sub>opt</sub> Name ArgumentsDefinition<sub>opt</sub> **:** TypeDirectives<sub>\[Const\] opt</sub>
///
/// Detailed documentation can be found in [GraphQL spec](https://spec.graphql.org/draft/#sec-The-__Field-Type).
///
/// ### Example
/// ```rust
/// use sdl_encoder::{Type_, Field, InputValue};
///
/// let ty_1 = Type_::NamedType {
///     name: "CatBreed".to_string(),
/// };
///
/// let mut field = Field::new("cat".to_string(), ty_1);
///
/// let value_1 = Type_::NamedType {
///     name: "CatBreed".to_string(),
/// };
///
/// let arg = InputValue::new("breed".to_string(), value_1);
///
/// field.arg(arg);
///
/// assert_eq!(
///     field.to_string(),
///     r#"  cat(breed: CatBreed): CatBreed"#
/// );
/// ```
#[derive(Debug, PartialEq, Clone)]
pub struct Field {
    // Name must return a String.
    name: String,
    // Description may return a String.
    description: Option<String>,
    // Args returns a List of __InputValue representing the arguments this field accepts.
    args: Vec<InputValue>,
    // Type must return a __Type that represents the type of value returned by this field.
    type_: Type_,
    // Deprecated returns true if this field should no longer be used, otherwise false.
    is_deprecated: bool,
    // Deprecation reason optionally provides a reason why this field is deprecated.
    deprecation_reason: Option<String>,
}

impl Field {
    /// Create a new instance of Field.
    pub fn new(name: String, type_: Type_) -> Self {
        Self {
            description: None,
            name,
            type_,
            args: Vec::new(),
            is_deprecated: false,
            deprecation_reason: None,
        }
    }

    /// Set the Field's description.
    pub fn description(&mut self, description: Option<String>) {
        self.description = description;
    }

    /// Set the Field's deprecation properties.
    pub fn deprecated(&mut self, reason: Option<String>) {
        self.is_deprecated = true;
        self.deprecation_reason = reason;
    }

    /// Set the Field's arguments.
    pub fn arg(&mut self, arg: InputValue) {
        self.args.push(arg);
    }
}

impl Display for Field {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(description) = &self.description {
            // Let's indent description on a field level for now, as all fields
            // are always on the same level and are indented by 2 spaces.
            //
            // We are also determing on whether to have description formatted as
            // a multiline comment based on whether or not it already includes a
            // \n.
            match description.contains('\n') {
                true => writeln!(f, "  \"\"\"\n  {}\n  \"\"\"", description)?,
                false => writeln!(f, "  \"\"\"{}\"\"\"", description)?,
            }
        }

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

        if self.is_deprecated {
            write!(f, " @deprecated")?;
            // Just in case deprecated field is ever used without a reason,
            // let's properly unwrap this Option.
            if let Some(reason) = &self.deprecation_reason {
                write!(f, "(reason: \"{}\")", reason)?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn it_encodes_simple_fields() {
        let ty_1 = Type_::NamedType {
            name: "SpaceProgram".to_string(),
        };

        let ty_2 = Type_::List { ty: Box::new(ty_1) };
        let ty_3 = Type_::NonNull { ty: Box::new(ty_2) };
        let field = Field::new("spaceCat".to_string(), ty_3);

        assert_eq!(field.to_string(), r#"  spaceCat: [SpaceProgram]!"#);
    }

    #[test]
    fn it_encodes_fields_with_deprecation() {
        let ty_1 = Type_::NamedType {
            name: "SpaceProgram".to_string(),
        };

        let ty_2 = Type_::List { ty: Box::new(ty_1) };
        let mut field = Field::new("cat".to_string(), ty_2);
        field.description(Some("Very good cats".to_string()));
        field.deprecated(Some("Cats are no longer sent to space.".to_string()));

        assert_eq!(
            field.to_string(),
            r#"  """Very good cats"""
  cat: [SpaceProgram] @deprecated(reason: "Cats are no longer sent to space.")"#
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
        let mut field = Field::new("spaceCat".to_string(), ty_4);
        field.description(Some("Very good space cats".to_string()));

        assert_eq!(
            field.to_string(),
            r#"  """Very good space cats"""
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
        let mut field = Field::new("spaceCat".to_string(), ty_4);
        field.description(Some("Very good space cats".to_string()));

        let value_1 = Type_::NamedType {
            name: "SpaceProgram".to_string(),
        };

        let value_2 = Type_::List {
            ty: Box::new(value_1),
        };
        let mut arg = InputValue::new("cat".to_string(), value_2);
        arg.deprecated(Some("Cats are no longer sent to space.".to_string()));
        field.arg(arg);

        assert_eq!(
            field.to_string(),
            r#"  """Very good space cats"""
  spaceCat(cat: [SpaceProgram] @deprecated(reason: "Cats are no longer sent to space.")): [SpaceProgram!]!"#
        );
    }
}
