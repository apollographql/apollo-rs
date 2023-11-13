use crate::ast::InvalidNameError;
use crate::schema::Name;
use std::fmt;
use std::str::FromStr;

/// Create a static coordinate at compile time.
///
/// ```rust
/// use apollo_compiler::coord;
///
/// assert_eq!(coord!(@directive).to_string(), "@directive");
/// assert_eq!(coord!(@directive(arg:)).to_string(), "@directive(arg:)");
/// assert_eq!(coord!(Type).to_string(), "Type");
/// assert_eq!(coord!(Type.field).to_string(), "Type.field");
/// assert_eq!(coord!(Type.field(arg:)).to_string(), "Type.field(arg:)");
/// ```
#[macro_export]
macro_rules! coord {
    ( @ $name:ident ) => {
        $crate::coordinate::DirectiveCoordinate($crate::name!($name))
    };
    ( @ $name:ident ( $arg:ident : ) ) => {
        $crate::coordinate::DirectiveArgumentCoordinate($crate::name!($name), $crate::name!($arg))
    };
    ( $name:ident ) => {
        $crate::coordinate::TypeCoordinate($crate::name!($name))
    };
    ( $name:ident . $field:ident ) => {
        $crate::coordinate::FieldCoordinate($crate::name!($name), $crate::name!($field))
    };
    ( $name:ident . $field:ident ( $arg:ident : ) ) => {
        $crate::coordinate::FieldArgumentCoordinate(
            $crate::name!($name),
            $crate::name!($field),
            $crate::name!($arg),
        )
    };
}

/// A schema coordinate targeting a type definition: `Type`.
///
/// # Example
/// ```
/// use apollo_compiler::coordinate::TypeCoordinate;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// assert_eq!(TypeCoordinate("Type".try_into()?).to_string(), "Type");
/// # Ok(()) }
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TypeCoordinate(pub Name);

/// A schema coordinate targeting a field definition: `Type.field`.
///
/// # Example
/// ```
/// use apollo_compiler::coordinate::FieldCoordinate;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// assert_eq!(FieldCoordinate("Type".try_into()?, "field".try_into()?).to_string(), "Type.field");
/// # Ok(()) }
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FieldCoordinate(pub Name, pub Name);

/// A schema coordinate targeting a field argument definition: `Type.field(argument:)`.
///
/// # Example
/// ```
/// use apollo_compiler::coordinate::FieldArgumentCoordinate;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// assert_eq!(FieldArgumentCoordinate("Type".try_into()?, "field".try_into()?, "argument".try_into()?).to_string(), "Type.field(argument:)");
/// # Ok(()) }
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FieldArgumentCoordinate(pub Name, pub Name, pub Name);

/// A schema coordinate targeting a directive definition: `@directive`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DirectiveCoordinate(pub Name);

/// A schema coordinate targeting a directive argument definition: `@directive(argument:)`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DirectiveArgumentCoordinate(pub Name, pub Name);

/// Any schema coordinate.
///
/// # Example
/// ```
/// use apollo_compiler::coordinate::{SchemaCoordinate, FieldArgumentCoordinate};
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let coord: SchemaCoordinate = "Type.field(argument:)".parse().unwrap();
/// assert_eq!(coord, SchemaCoordinate::FieldArgument(
///     FieldArgumentCoordinate("Type".try_into()?, "field".try_into()?, "argument".try_into()?)
/// ));
/// # Ok(()) }
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SchemaCoordinate {
    Type(TypeCoordinate),
    Field(FieldCoordinate),
    FieldArgument(FieldArgumentCoordinate),
    Directive(DirectiveCoordinate),
    DirectiveArgument(DirectiveArgumentCoordinate),
}

#[derive(Debug, Clone, thiserror::Error)]
#[non_exhaustive]
pub enum SchemaCoordinateParseError {
    #[error("invalid schema coordinate")]
    InvalidFormat,
    #[error(transparent)]
    InvalidName(#[from] InvalidNameError),
}

impl TypeCoordinate {
    /// Create a schema coordinate that points to a field on this type.
    pub fn field(self, field: Name) -> FieldCoordinate {
        FieldCoordinate(self.0, field)
    }
}

impl FromStr for TypeCoordinate {
    type Err = SchemaCoordinateParseError;
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        Ok(Self(Name::try_from(input)?))
    }
}

impl FieldCoordinate {
    /// Create a schema coordinate that points to the type this field is on.
    pub fn type_(self) -> TypeCoordinate {
        TypeCoordinate(self.0)
    }

    /// Create a schema coordinate that points to an argument on this field.
    pub fn argument(self, argument: Name) -> FieldArgumentCoordinate {
        FieldArgumentCoordinate(self.0, self.1, argument)
    }
}

impl FromStr for FieldCoordinate {
    type Err = SchemaCoordinateParseError;
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let Some((type_name, field)) = input.split_once('.') else {
            return Err(SchemaCoordinateParseError::InvalidFormat);
        };
        Ok(Self(Name::try_from(type_name)?, Name::try_from(field)?))
    }
}

impl FieldArgumentCoordinate {
    /// Create a schema coordinate that points to the type this argument is defined in.
    pub fn type_(self) -> TypeCoordinate {
        TypeCoordinate(self.0)
    }

    /// Create a schema coordinate that points to the field this argument is defined in.
    pub fn field(self) -> FieldCoordinate {
        FieldCoordinate(self.0, self.1)
    }
}

impl FromStr for FieldArgumentCoordinate {
    type Err = SchemaCoordinateParseError;
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let Some((field, rest)) = input.split_once('(') else {
            return Err(SchemaCoordinateParseError::InvalidFormat);
        };
        let field = FieldCoordinate::from_str(field)?;

        let Some((argument, ")")) = rest.split_once(':') else {
            return Err(SchemaCoordinateParseError::InvalidFormat);
        };
        Ok(field.argument(Name::try_from(argument)?))
    }
}

impl DirectiveCoordinate {
    /// Create a schema coordinate that points to an argument of this directive.
    pub fn argument(self, argument: Name) -> DirectiveArgumentCoordinate {
        DirectiveArgumentCoordinate(self.0, argument)
    }
}

impl FromStr for DirectiveCoordinate {
    type Err = SchemaCoordinateParseError;
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        if let Some(directive) = input.strip_prefix('@') {
            Ok(Self(Name::try_from(directive)?))
        } else {
            Err(SchemaCoordinateParseError::InvalidFormat)
        }
    }
}

impl DirectiveArgumentCoordinate {
    /// Create a schema coordinate that points to the directive this argument is defined in.
    pub fn directive(self) -> DirectiveCoordinate {
        DirectiveCoordinate(self.0)
    }
}

impl FromStr for DirectiveArgumentCoordinate {
    type Err = SchemaCoordinateParseError;
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let Some((directive, rest)) = input.split_once('(') else {
            return Err(SchemaCoordinateParseError::InvalidFormat);
        };
        let directive = DirectiveCoordinate::from_str(directive)?;

        let Some((argument, ")")) = rest.split_once(':') else {
            return Err(SchemaCoordinateParseError::InvalidFormat);
        };
        Ok(directive.argument(Name::try_from(argument)?))
    }
}

impl fmt::Display for TypeCoordinate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self(type_name) = self;
        write!(f, "{type_name}")
    }
}

impl fmt::Display for FieldCoordinate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self(type_name, field) = self;
        write!(f, "{type_name}.{field}")
    }
}

impl fmt::Display for FieldArgumentCoordinate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self(type_name, field, argument) = self;
        write!(f, "{type_name}.{field}({argument}:)")
    }
}

impl fmt::Display for DirectiveCoordinate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self(directive) = self;
        write!(f, "@{directive}")
    }
}

impl fmt::Display for DirectiveArgumentCoordinate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self(directive, argument) = self;
        write!(f, "@{directive}({argument}:)")
    }
}

impl FromStr for SchemaCoordinate {
    type Err = SchemaCoordinateParseError;
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        if input.starts_with('@') {
            DirectiveArgumentCoordinate::from_str(input)
                .map(Self::DirectiveArgument)
                .or_else(|_| DirectiveCoordinate::from_str(input).map(Self::Directive))
        } else {
            FieldArgumentCoordinate::from_str(input)
                .map(Self::FieldArgument)
                .or_else(|_| FieldCoordinate::from_str(input).map(Self::Field))
                .or_else(|_| TypeCoordinate::from_str(input).map(Self::Type))
        }
    }
}

impl fmt::Display for SchemaCoordinate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Type(inner) => inner.fmt(f),
            Self::Field(inner) => inner.fmt(f),
            Self::FieldArgument(inner) => inner.fmt(f),
            Self::Directive(inner) => inner.fmt(f),
            Self::DirectiveArgument(inner) => inner.fmt(f),
        }
    }
}
