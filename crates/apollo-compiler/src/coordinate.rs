//! Parsing and printing for schema coordinates as described in [the RFC].
//!
//! Schema coordinates uniquely point to an item defined in a schema.
//!
//! [the RFC]: https://github.com/graphql/graphql-wg/blob/main/rfcs/SchemaCoordinates.md

use crate::schema::Component;
use crate::schema::DirectiveDefinition;
use crate::schema::EnumValueDefinition;
use crate::schema::ExtendedType;
use crate::schema::FieldDefinition;
use crate::schema::InputValueDefinition;
use crate::schema::NamedType;
use crate::schema::Schema;
use crate::InvalidNameError;
use crate::Name;
use crate::Node;
use std::fmt;
use std::str::FromStr;

/// Create a [`DirectiveCoordinate`], [`DirectiveArgumentCoordinate`],
/// [`TypeCoordinate`], [`TypeAttributeCoordinate`],
/// or [`FieldArgumentCoordinate`] at compile time.
///
/// ```rust
/// use apollo_compiler::coord;
///
/// assert_eq!(coord!(@directive).to_string(), "@directive");
/// assert_eq!(coord!(@directive(arg:)).to_string(), "@directive(arg:)");
/// assert_eq!(coord!(Type).to_string(), "Type");
/// assert_eq!(coord!(Type.field).to_string(), "Type.field");
/// assert_eq!(coord!(Type.field(arg:)).to_string(), "Type.field(arg:)");
/// assert_eq!(coord!(EnumType.ENUM_VALUE).to_string(), "EnumType.ENUM_VALUE");
/// ```
///
/// All possible return types of this macro implement
/// [`From`]`<`[`SchemaCoordinate`]`>` so they can be converted using `.into()`:
///
/// ```
/// use apollo_compiler::coord;
/// use apollo_compiler::coordinate::SchemaCoordinate;
///
/// let _: SchemaCoordinate = coord!(Query).into();
/// ```
#[macro_export]
macro_rules! coord {
    ( @ $name:ident ) => {
        $crate::coordinate::DirectiveCoordinate {
            directive: $crate::name!($name),
        }
    };
    ( @ $name:ident ( $arg:ident : ) ) => {
        $crate::coordinate::DirectiveArgumentCoordinate {
            directive: $crate::name!($name),
            argument: $crate::name!($arg),
        }
    };
    ( $name:ident ) => {
        $crate::coordinate::TypeCoordinate {
            ty: $crate::name!($name),
        }
    };
    ( $name:ident . $attribute:ident ) => {
        $crate::coordinate::TypeAttributeCoordinate {
            ty: $crate::name!($name),
            attribute: $crate::name!($attribute),
        }
    };
    ( $name:ident . $field:ident ( $arg:ident : ) ) => {
        $crate::coordinate::FieldArgumentCoordinate {
            ty: $crate::name!($name),
            field: $crate::name!($field),
            argument: $crate::name!($arg),
        }
    };
}

/// A schema coordinate targeting a type definition: `Type`.
///
/// # Example
/// ```
/// use apollo_compiler::name;
/// use apollo_compiler::coordinate::TypeCoordinate;
///
/// assert_eq!(TypeCoordinate { ty: name!("Type") }.to_string(), "Type");
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TypeCoordinate {
    pub ty: NamedType,
}

/// A schema coordinate targeting a field definition or an enum value: `Type.field`, `Enum.VALUE`.
///
/// Type attribute coordinate syntax can refer to object or interface field definitions, input
/// field definitions, and enum values. [`TypeAttributeCoordinate::lookup`] returns an enum to
/// account for those possibilities. To look up a specific kind of type attribute, there are
/// convenience methods:
/// - [`TypeAttributeCoordinate::lookup_field`] for object or interface fields
/// - [`TypeAttributeCoordinate::lookup_input_field`] for input fields
/// - [`TypeAttributeCoordinate::lookup_enum_value`] for enum values
///
/// # Example
/// ```
/// use apollo_compiler::name;
/// use apollo_compiler::coordinate::TypeAttributeCoordinate;
///
/// assert_eq!(TypeAttributeCoordinate {
///     ty: name!("Type"),
///     attribute: name!("field"),
/// }.to_string(), "Type.field");
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TypeAttributeCoordinate {
    pub ty: NamedType,
    pub attribute: Name,
}

/// A schema coordinate targeting a field argument definition: `Type.field(argument:)`.
///
/// # Example
/// ```
/// use apollo_compiler::name;
/// use apollo_compiler::coordinate::FieldArgumentCoordinate;
///
/// assert_eq!(FieldArgumentCoordinate {
///     ty: name!("Type"),
///     field: name!("field"),
///     argument: name!("argument"),
/// }.to_string(), "Type.field(argument:)");
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FieldArgumentCoordinate {
    pub ty: NamedType,
    pub field: Name,
    pub argument: Name,
}

/// A schema coordinate targeting a directive definition: `@directive`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DirectiveCoordinate {
    pub directive: Name,
}

/// A schema coordinate targeting a directive argument definition: `@directive(argument:)`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DirectiveArgumentCoordinate {
    pub directive: Name,
    pub argument: Name,
}

/// Any schema coordinate.
///
/// # Example
/// ```
/// use apollo_compiler::name;
/// use apollo_compiler::coordinate::{SchemaCoordinate, FieldArgumentCoordinate};
///
/// let coord: SchemaCoordinate = "Type.field(argument:)".parse().unwrap();
/// assert_eq!(coord, SchemaCoordinate::FieldArgument(
///     FieldArgumentCoordinate {
///         ty: name!("Type"),
///         field: name!("field"),
///         argument: name!("argument"),
///     },
/// ));
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SchemaCoordinate {
    Type(TypeCoordinate),
    TypeAttribute(TypeAttributeCoordinate),
    FieldArgument(FieldArgumentCoordinate),
    Directive(DirectiveCoordinate),
    DirectiveArgument(DirectiveArgumentCoordinate),
}

/// Errors that can occur while parsing a schema coordinate.
#[derive(Debug, Clone, thiserror::Error)]
#[non_exhaustive]
pub enum SchemaCoordinateParseError {
    /// Invalid format, eg. unexpected characters
    #[error("invalid schema coordinate")]
    InvalidFormat,
    /// A name part contains invalid characters
    #[error(transparent)]
    InvalidName(#[from] InvalidNameError),
}

/// The error type for [`SchemaCoordinate::lookup`] and other `lookup*` methods.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum SchemaLookupError<'coord, 'schema> {
    /// The requested type does not exist in the schema.
    #[error("type `{0}` does not exist")]
    MissingType(&'coord NamedType),
    /// The requested field or enum value does not exist on its type.
    #[error("type does not have attribute `{0}`")]
    MissingAttribute(&'coord Name),
    /// The requested argument can not be looked up because its type does not support arguments.
    #[error("type attribute `{0}` is not a field and can not have arguments")]
    InvalidArgumentAttribute(&'coord Name),
    /// The requested argument does not exist on its field or directive.
    #[error("field or directive does not have argument `{0}`")]
    MissingArgument(&'coord Name),
    /// The requested field or enum value can not be looked up because its type does not support
    /// fields.
    #[error("type does not have attributes")]
    InvalidType(&'schema ExtendedType),
}

/// Return type of [`TypeAttributeCoordinate::lookup`], for coordinates of the form `Type.field`.
#[derive(Debug, Clone, PartialEq, Eq)]
// Should this be non-exhaustive? Allows for future extension should unions ever be added.
#[non_exhaustive]
pub enum TypeAttributeLookup<'schema> {
    Field(&'schema Component<FieldDefinition>),
    InputField(&'schema Component<InputValueDefinition>),
    EnumValue(&'schema Component<EnumValueDefinition>),
}

/// Return type of [`SchemaCoordinate::lookup`].
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SchemaCoordinateLookup<'schema> {
    Type(&'schema ExtendedType),
    Directive(&'schema Node<DirectiveDefinition>),
    Field(&'schema Component<FieldDefinition>),
    InputField(&'schema Component<InputValueDefinition>),
    EnumValue(&'schema Component<EnumValueDefinition>),
    Argument(&'schema Node<InputValueDefinition>),
}

impl TypeCoordinate {
    /// Create a schema coordinate that points to an attribute on this type.
    ///
    /// For object types and interfaces, the resulting coordinate points to a field. For enums, the
    /// resulting coordinate points to a value.
    pub fn with_attribute(&self, attribute: Name) -> TypeAttributeCoordinate {
        TypeAttributeCoordinate {
            ty: self.ty.clone(),
            attribute,
        }
    }

    fn lookup_ref<'coord, 'schema>(
        ty: &'coord NamedType,
        schema: &'schema Schema,
    ) -> Result<&'schema ExtendedType, SchemaLookupError<'coord, 'schema>> {
        schema
            .types
            .get(ty)
            .ok_or(SchemaLookupError::MissingType(ty))
    }

    /// Look up this type coordinate in a schema.
    pub fn lookup<'coord, 'schema>(
        &'coord self,
        schema: &'schema Schema,
    ) -> Result<&'schema ExtendedType, SchemaLookupError<'coord, 'schema>> {
        Self::lookup_ref(&self.ty, schema)
    }
}

impl FromStr for TypeCoordinate {
    type Err = SchemaCoordinateParseError;
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        Ok(Self {
            ty: NamedType::try_from(input)?,
        })
    }
}

impl TypeAttributeCoordinate {
    /// Create a schema coordinate that points to the type this attribute is part of.
    pub fn type_coordinate(&self) -> TypeCoordinate {
        TypeCoordinate {
            ty: self.ty.clone(),
        }
    }

    /// Assume this attribute is a field, and create a schema coordinate that points to an argument on this field.
    pub fn with_argument(&self, argument: Name) -> FieldArgumentCoordinate {
        FieldArgumentCoordinate {
            ty: self.ty.clone(),
            field: self.attribute.clone(),
            argument,
        }
    }

    fn lookup_ref<'coord, 'schema>(
        ty: &'coord NamedType,
        attribute: &'coord Name,
        schema: &'schema Schema,
    ) -> Result<TypeAttributeLookup<'schema>, SchemaLookupError<'coord, 'schema>> {
        let ty = TypeCoordinate::lookup_ref(ty, schema)?;
        match ty {
            ExtendedType::Enum(enum_) => enum_
                .values
                .get(attribute)
                .ok_or(SchemaLookupError::MissingAttribute(attribute))
                .map(TypeAttributeLookup::EnumValue),
            ExtendedType::InputObject(input_object) => input_object
                .fields
                .get(attribute)
                .ok_or(SchemaLookupError::MissingAttribute(attribute))
                .map(TypeAttributeLookup::InputField),
            ExtendedType::Object(object) => object
                .fields
                .get(attribute)
                .ok_or(SchemaLookupError::MissingAttribute(attribute))
                .map(TypeAttributeLookup::Field),
            ExtendedType::Interface(interface) => interface
                .fields
                .get(attribute)
                .ok_or(SchemaLookupError::MissingAttribute(attribute))
                .map(TypeAttributeLookup::Field),
            ExtendedType::Union(_) | ExtendedType::Scalar(_) => {
                Err(SchemaLookupError::InvalidType(ty))
            }
        }
    }

    /// Look up this type attribute in a schema.
    pub fn lookup<'coord, 'schema>(
        &'coord self,
        schema: &'schema Schema,
    ) -> Result<TypeAttributeLookup<'schema>, SchemaLookupError<'coord, 'schema>> {
        Self::lookup_ref(&self.ty, &self.attribute, schema)
    }

    /// Look up this field definition in a schema. If the attribute does not refer to an object or
    /// interface field, returns `SchemaLookupError::InvalidType`.
    pub fn lookup_field<'coord, 'schema>(
        &'coord self,
        schema: &'schema Schema,
    ) -> Result<&'schema Component<FieldDefinition>, SchemaLookupError<'coord, 'schema>> {
        let ty = TypeCoordinate::lookup_ref(&self.ty, schema)?;
        match ty {
            ExtendedType::Object(object) => object
                .fields
                .get(&self.attribute)
                .ok_or(SchemaLookupError::MissingAttribute(&self.attribute)),
            ExtendedType::Interface(interface) => interface
                .fields
                .get(&self.attribute)
                .ok_or(SchemaLookupError::MissingAttribute(&self.attribute)),
            _ => Err(SchemaLookupError::InvalidType(ty)),
        }
    }

    /// Look up this input field definition in a schema. If the attribute does not refer to an
    /// input field, returns `SchemaLookupError::InvalidType`.
    pub fn lookup_input_field<'coord, 'schema>(
        &'coord self,
        schema: &'schema Schema,
    ) -> Result<&'schema Component<InputValueDefinition>, SchemaLookupError<'coord, 'schema>> {
        let ty = TypeCoordinate::lookup_ref(&self.ty, schema)?;
        match ty {
            ExtendedType::InputObject(object) => object
                .fields
                .get(&self.attribute)
                .ok_or(SchemaLookupError::MissingAttribute(&self.attribute)),
            _ => Err(SchemaLookupError::InvalidType(ty)),
        }
    }

    /// Look up this enum value definition in a schema. If the attribute does not refer to an
    /// enum, returns `SchemaLookupError::InvalidType`.
    pub fn lookup_enum_value<'coord, 'schema>(
        &'coord self,
        schema: &'schema Schema,
    ) -> Result<&'schema Component<EnumValueDefinition>, SchemaLookupError<'coord, 'schema>> {
        let ty = TypeCoordinate::lookup_ref(&self.ty, schema)?;
        match ty {
            ExtendedType::Enum(enum_) => enum_
                .values
                .get(&self.attribute)
                .ok_or(SchemaLookupError::MissingAttribute(&self.attribute)),
            _ => Err(SchemaLookupError::InvalidType(ty)),
        }
    }
}

impl FromStr for TypeAttributeCoordinate {
    type Err = SchemaCoordinateParseError;
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let Some((type_name, field)) = input.split_once('.') else {
            return Err(SchemaCoordinateParseError::InvalidFormat);
        };
        Ok(Self {
            ty: NamedType::try_from(type_name)?,
            attribute: Name::try_from(field)?,
        })
    }
}

impl FieldArgumentCoordinate {
    /// Create a schema coordinate that points to the type this argument is defined in.
    pub fn type_coordinate(&self) -> TypeCoordinate {
        TypeCoordinate {
            ty: self.ty.clone(),
        }
    }

    /// Create a schema coordinate that points to the field this argument is defined in.
    pub fn field_coordinate(&self) -> TypeAttributeCoordinate {
        TypeAttributeCoordinate {
            ty: self.ty.clone(),
            attribute: self.field.clone(),
        }
    }

    fn lookup_ref<'coord, 'schema>(
        ty: &'coord NamedType,
        field: &'coord Name,
        argument: &'coord Name,
        schema: &'schema Schema,
    ) -> Result<&'schema Node<InputValueDefinition>, SchemaLookupError<'coord, 'schema>> {
        match TypeAttributeCoordinate::lookup_ref(ty, field, schema)? {
            TypeAttributeLookup::Field(field) => field
                .argument_by_name(argument)
                .ok_or(SchemaLookupError::MissingArgument(argument)),
            _ => Err(SchemaLookupError::InvalidArgumentAttribute(field)),
        }
    }

    /// Look up this argument definition in a schema.
    pub fn lookup<'coord, 'schema>(
        &'coord self,
        schema: &'schema Schema,
    ) -> Result<&'schema Node<InputValueDefinition>, SchemaLookupError<'coord, 'schema>> {
        Self::lookup_ref(&self.ty, &self.field, &self.argument, schema)
    }
}

impl FromStr for FieldArgumentCoordinate {
    type Err = SchemaCoordinateParseError;
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let Some((field, rest)) = input.split_once('(') else {
            return Err(SchemaCoordinateParseError::InvalidFormat);
        };
        let field = TypeAttributeCoordinate::from_str(field)?;

        let Some((argument, ")")) = rest.split_once(':') else {
            return Err(SchemaCoordinateParseError::InvalidFormat);
        };
        Ok(Self {
            ty: field.ty,
            field: field.attribute,
            argument: Name::try_from(argument)?,
        })
    }
}

impl DirectiveCoordinate {
    /// Create a schema coordinate that points to an argument of this directive.
    pub fn with_argument(&self, argument: Name) -> DirectiveArgumentCoordinate {
        DirectiveArgumentCoordinate {
            directive: self.directive.clone(),
            argument,
        }
    }

    fn lookup_ref<'coord, 'schema>(
        directive: &'coord Name,
        schema: &'schema Schema,
    ) -> Result<&'schema Node<DirectiveDefinition>, SchemaLookupError<'coord, 'schema>> {
        schema
            .directive_definitions
            .get(directive)
            .ok_or(SchemaLookupError::MissingType(directive))
    }

    /// Look up this directive in a schema.
    pub fn lookup<'coord, 'schema>(
        &'coord self,
        schema: &'schema Schema,
    ) -> Result<&'schema Node<DirectiveDefinition>, SchemaLookupError<'coord, 'schema>> {
        Self::lookup_ref(&self.directive, schema)
    }
}

impl From<Name> for DirectiveCoordinate {
    fn from(directive: Name) -> Self {
        Self { directive }
    }
}

impl FromStr for DirectiveCoordinate {
    type Err = SchemaCoordinateParseError;
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        if let Some(directive) = input.strip_prefix('@') {
            Ok(Self {
                directive: Name::try_from(directive)?,
            })
        } else {
            Err(SchemaCoordinateParseError::InvalidFormat)
        }
    }
}

impl DirectiveArgumentCoordinate {
    /// Create a schema coordinate that points to the directive this argument is defined in.
    pub fn directive_coordinate(&self) -> DirectiveCoordinate {
        DirectiveCoordinate {
            directive: self.directive.clone(),
        }
    }

    fn lookup_ref<'coord, 'schema>(
        directive: &'coord Name,
        argument: &'coord Name,
        schema: &'schema Schema,
    ) -> Result<&'schema Node<InputValueDefinition>, SchemaLookupError<'coord, 'schema>> {
        DirectiveCoordinate::lookup_ref(directive, schema)?
            .argument_by_name(argument)
            .ok_or(SchemaLookupError::MissingArgument(argument))
    }

    /// Look up this directive argument in a schema.
    pub fn lookup<'coord, 'schema>(
        &'coord self,
        schema: &'schema Schema,
    ) -> Result<&'schema Node<InputValueDefinition>, SchemaLookupError<'coord, 'schema>> {
        Self::lookup_ref(&self.directive, &self.argument, schema)
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
        Ok(Self {
            directive: directive.directive,
            argument: Name::try_from(argument)?,
        })
    }
}

impl<'schema> From<&'schema ExtendedType> for SchemaCoordinateLookup<'schema> {
    fn from(inner: &'schema ExtendedType) -> Self {
        Self::Type(inner)
    }
}

impl<'schema> From<&'schema Node<DirectiveDefinition>> for SchemaCoordinateLookup<'schema> {
    fn from(inner: &'schema Node<DirectiveDefinition>) -> Self {
        Self::Directive(inner)
    }
}

impl<'schema> From<&'schema Component<FieldDefinition>> for SchemaCoordinateLookup<'schema> {
    fn from(inner: &'schema Component<FieldDefinition>) -> Self {
        Self::Field(inner)
    }
}

impl<'schema> From<&'schema Component<InputValueDefinition>> for SchemaCoordinateLookup<'schema> {
    fn from(inner: &'schema Component<InputValueDefinition>) -> Self {
        Self::InputField(inner)
    }
}

impl<'schema> From<&'schema Component<EnumValueDefinition>> for SchemaCoordinateLookup<'schema> {
    fn from(inner: &'schema Component<EnumValueDefinition>) -> Self {
        Self::EnumValue(inner)
    }
}

impl<'schema> From<TypeAttributeLookup<'schema>> for SchemaCoordinateLookup<'schema> {
    fn from(attr: TypeAttributeLookup<'schema>) -> Self {
        match attr {
            TypeAttributeLookup::Field(field) => SchemaCoordinateLookup::Field(field),
            TypeAttributeLookup::InputField(field) => SchemaCoordinateLookup::InputField(field),
            TypeAttributeLookup::EnumValue(field) => SchemaCoordinateLookup::EnumValue(field),
        }
    }
}

impl<'schema> From<&'schema Node<InputValueDefinition>> for SchemaCoordinateLookup<'schema> {
    fn from(inner: &'schema Node<InputValueDefinition>) -> Self {
        Self::Argument(inner)
    }
}

impl SchemaCoordinate {
    /// Look up this coordinate in a schema.
    pub fn lookup<'coord, 'schema>(
        &'coord self,
        schema: &'schema Schema,
    ) -> Result<SchemaCoordinateLookup<'schema>, SchemaLookupError<'coord, 'schema>> {
        match self {
            SchemaCoordinate::Type(coordinate) => coordinate.lookup(schema).map(Into::into),
            SchemaCoordinate::TypeAttribute(coordinate) => {
                coordinate.lookup(schema).map(Into::into)
            }
            SchemaCoordinate::FieldArgument(coordinate) => {
                coordinate.lookup(schema).map(Into::into)
            }
            SchemaCoordinate::Directive(coordinate) => coordinate.lookup(schema).map(Into::into),
            SchemaCoordinate::DirectiveArgument(coordinate) => {
                coordinate.lookup(schema).map(Into::into)
            }
        }
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
                .or_else(|_| TypeAttributeCoordinate::from_str(input).map(Self::TypeAttribute))
                .or_else(|_| TypeCoordinate::from_str(input).map(Self::Type))
        }
    }
}

impl From<TypeCoordinate> for SchemaCoordinate {
    fn from(inner: TypeCoordinate) -> Self {
        Self::Type(inner)
    }
}

impl From<TypeAttributeCoordinate> for SchemaCoordinate {
    fn from(inner: TypeAttributeCoordinate) -> Self {
        Self::TypeAttribute(inner)
    }
}

impl From<FieldArgumentCoordinate> for SchemaCoordinate {
    fn from(inner: FieldArgumentCoordinate) -> Self {
        Self::FieldArgument(inner)
    }
}

impl From<DirectiveCoordinate> for SchemaCoordinate {
    fn from(inner: DirectiveCoordinate) -> Self {
        Self::Directive(inner)
    }
}

impl From<DirectiveArgumentCoordinate> for SchemaCoordinate {
    fn from(inner: DirectiveArgumentCoordinate) -> Self {
        Self::DirectiveArgument(inner)
    }
}

impl fmt::Display for TypeCoordinate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { ty } = self;
        write!(f, "{ty}")
    }
}

impl fmt::Display for TypeAttributeCoordinate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self {
            ty,
            attribute: field,
        } = self;
        write!(f, "{ty}.{field}")
    }
}

impl fmt::Display for FieldArgumentCoordinate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self {
            ty,
            field,
            argument,
        } = self;
        write!(f, "{ty}.{field}({argument}:)")
    }
}

impl fmt::Display for DirectiveCoordinate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { directive } = self;
        write!(f, "@{directive}")
    }
}

impl fmt::Display for DirectiveArgumentCoordinate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self {
            directive,
            argument,
        } = self;
        write!(f, "@{directive}({argument}:)")
    }
}

impl fmt::Display for SchemaCoordinate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Type(inner) => inner.fmt(f),
            Self::TypeAttribute(inner) => inner.fmt(f),
            Self::FieldArgument(inner) => inner.fmt(f),
            Self::Directive(inner) => inner.fmt(f),
            Self::DirectiveArgument(inner) => inner.fmt(f),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_coordinates() {
        SchemaCoordinate::from_str("Type\\.field(arg:)").expect_err("invalid character");
        SchemaCoordinate::from_str("@directi^^ve").expect_err("invalid character");
        SchemaCoordinate::from_str("@directi@ve").expect_err("invalid character");
        SchemaCoordinate::from_str("@  spaces  ").expect_err("invalid character");

        SchemaCoordinate::from_str("@(:)").expect_err("directive argument syntax without names");
        SchemaCoordinate::from_str("@dir(:)")
            .expect_err("directive argument syntax without argument name");
        SchemaCoordinate::from_str("@(arg:)")
            .expect_err("directive argument syntax without directive name");

        SchemaCoordinate::from_str("Type.")
            .expect_err("type attribute syntax without attribute name");
        SchemaCoordinate::from_str(".field").expect_err("type attribute syntax without type name");
        SchemaCoordinate::from_str("Type.field(:)")
            .expect_err("field argument syntax without field name");
        SchemaCoordinate::from_str("Type.field(arg)").expect_err("field argument syntax without :");
    }
}
