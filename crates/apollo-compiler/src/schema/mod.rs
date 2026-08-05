//! High-level representation of a GraphQL type system document a.k.a. schema.
//!
//! Compared to an [`ast::Document`] which follows closely the structure of GraphQL syntax,
//! a [`Schema`] is organized for semantics first:
//!
//! * Wherever something is meant to have a unique name (for example fields of a given object type),
//!   a collection is stored as [`IndexMap<Name, _>`] instead of [`Vec<_>`]
//!   in order to facilitate lookup by name while preserving source ordering.
//!
//! * Everything from [type system extensions] is stored
//!   together with corresponding “main” definitions,
//!   while still preserving extension origins with [`Node<_>`].
//!   so that most consumers don’t need to care about extensions at all,
//!   (For example, some directives can be applied to an object type extensions to affect
//!   fields defined in the same extension but not other fields of the object type.)
//!   See [`Component`].
//!
//! [type system extensions]: https://spec.graphql.org/September2025/#sec-Type-System-Extensions
//!
//! In some cases like [`SchemaDefinition`], this module and the [`ast`] module
//! define different Rust types with the same names.
//! In other cases like [`Directive`] there is no data structure difference needed,
//! so this module reuses and publicly re-exports some Rust types from the [`ast`] module.
//!
//! ## “Build” errors
//!
//! As a result of how `Schema` is structured,
//! not all AST documents (even if filtering out executable definitions) can be fully represented:
//! creating a `Schema` can cause errors (on top of any potential syntax error)
//! for cases like name collisions.
//!
//! When such errors (or in [`Schema::parse`], syntax errors) happen,
//! a partial schema is returned together with a list of diagnostics.
//!
//! ## Structural sharing and mutation
//!
//! Many parts of a `Schema` are reference-counted with [`Node`] (like in AST) or [`Component`].
//! This allows sharing nodes between documents without cloning entire subtrees.
//! To modify a node or component,
//! the [`make_mut`][Node::make_mut] method provides copy-on-write semantics.
//!
//! ## Validation
//!
//! The [Type System] section of the GraphQL specification defines validation rules
//! beyond syntax errors and errors detected while constructing a `Schema`.
//! The [`validate`][Schema::validate] method returns either:
//!
//! * An immutable [`Valid<Schema>`] type wrapper, or
//! * The schema together with a list of diagnostics
//!
//! If there is no mutation needed between parsing and validation,
//! [`Schema::parse_and_validate`] does both in one step.
//!
//! [Type System]: https://spec.graphql.org/September2025/#sec-Type-System
//!
//! ## Serialization
//!
//! [`Schema`] and other types types implement [`Display`][std::fmt::Display]
//! and [`ToString`] by serializing to GraphQL syntax with a default configuration.
//! [`serialize`][Schema::serialize] methods return a builder
//! that has chaining methods for setting serialization configuration,
//! and also implements `Display` and `ToString`.

use crate::ast;
use crate::collections::HashMap;
use crate::collections::IndexMap;
use crate::collections::IndexSet;
use crate::name;
use crate::parser::FileId;
use crate::parser::Parser;
use crate::parser::SourceSpan;
use crate::ty;
use crate::validation::DiagnosticList;
use crate::validation::Valid;
use crate::validation::WithErrors;
pub use crate::Name;
use crate::Node;
use std::path::Path;
use std::sync::OnceLock;

mod component;
mod from_ast;
mod serialize;
pub(crate) mod validation;

pub use self::component::Component;
pub use self::component::ComponentName;
pub use self::component::ExtensionId;
pub use self::from_ast::SchemaBuilder;
pub use crate::ast::Directive;
pub use crate::ast::DirectiveDefinition;
pub use crate::ast::DirectiveList;
pub use crate::ast::DirectiveLocation;
pub use crate::ast::EnumValueDefinition;
pub use crate::ast::FieldDefinition;
pub use crate::ast::InputValueDefinition;
pub use crate::ast::NamedType;
pub use crate::ast::Type;
pub use crate::ast::Value;

/// High-level representation of a GraphQL type system document a.k.a. schema.
#[derive(Clone)]
pub struct Schema {
    /// Source files, if any, that were parsed to contribute to this schema.
    ///
    /// The schema (including parsed definitions) may have been modified since parsing.
    pub sources: crate::parser::SourceMap,

    /// The `schema` definition and its extensions, defining root operations
    pub schema_definition: Node<SchemaDefinition>,

    /// Built-in and explicit directive definitions
    pub directive_definitions: IndexMap<Name, Node<DirectiveDefinition>>,

    /// Definitions and extensions of all types relevant to a schema:
    ///
    /// * Explict types in parsed input files or added programatically.
    ///
    /// * [Schema-introspection](https://spec.graphql.org/September2025/#sec-Schema-Introspection)
    ///   types such as `__Schema`, `__Field`, etc.
    ///
    /// * When a `Schema` is initially created or parsed,
    ///   all [Built-in scalars](https://spec.graphql.org/September2025/#sec-Scalars.Built-in-Scalars).
    ///   After validation, the Rust `types` map in a `Valid<Schema>` only contains
    ///   built-in scalar definitions for scalars that are used in the schema.
    ///   We reflect in this Rust API the behavior of `__Schema.types` in GraphQL introspection.
    pub types: IndexMap<NamedType, ExtendedType>,

    /// Whether to validate default values of input fields and arguments
    /// against their types. Defaults to `true`.
    ///
    /// Set to `false` via [`SchemaBuilder::validate_default_values`]
    /// to accept schemas with mistyped default values.
    pub validate_default_values: bool,
}

/// The [`schema` definition](https://spec.graphql.org/September2025/#sec-Schema) and its extensions,
/// defining root operations
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SchemaDefinition {
    pub description: Option<Node<str>>,
    pub directives: DirectiveList,

    /// Name of the object type for the `query` root operation
    pub query: Option<Node<Name>>,

    /// Name of the object type for the `mutation` root operation
    pub mutation: Option<Node<Name>>,

    /// Name of the object type for the `subscription` root operation
    pub subscription: Option<Node<Name>>,
}

/// The definition of a named type, with all information from type extensions folded in.
///
/// The source location is that of the "main" definition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExtendedType {
    Scalar(Node<ScalarType>),
    Object(Node<ObjectType>),
    Interface(Node<InterfaceType>),
    Union(Node<UnionType>),
    Enum(Node<EnumType>),
    InputObject(Node<InputObjectType>),
}

/// The definition of a [scalar type](https://spec.graphql.org/September2025/#sec-Scalars),
/// with all information from type extensions folded in.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ScalarType {
    pub description: Option<Node<str>>,
    pub name: Name,
    pub directives: DirectiveList,
}

/// The definition of an [object type](https://spec.graphql.org/September2025/#sec-Objects),
/// with all information from type extensions folded in.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObjectType {
    pub description: Option<Node<str>>,
    pub name: Name,
    pub implements_interfaces: IndexSet<Node<Name>>,
    pub directives: DirectiveList,

    /// Explicit field definitions.
    ///
    /// When looking up a definition,
    /// consider using [`Schema::type_field`] instead to include meta-fields.
    pub fields: IndexMap<Name, Node<FieldDefinition>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InterfaceType {
    pub description: Option<Node<str>>,
    pub name: Name,
    pub implements_interfaces: IndexSet<Node<Name>>,

    pub directives: DirectiveList,

    /// Explicit field definitions.
    ///
    /// When looking up a definition,
    /// consider using [`Schema::type_field`] instead to include meta-fields.
    pub fields: IndexMap<Name, Node<FieldDefinition>>,
}

/// The definition of an [union type](https://spec.graphql.org/September2025/#sec-Unions),
/// with all information from type extensions folded in.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnionType {
    pub description: Option<Node<str>>,
    pub name: Name,
    pub directives: DirectiveList,

    /// * Key: name of a member object type
    /// * Value: which union type extension defined this implementation,
    ///   or `None` for the union type definition.
    pub members: IndexSet<Node<Name>>,
}

/// The definition of an [enum type](https://spec.graphql.org/September2025/#sec-Enums),
/// with all information from type extensions folded in.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnumType {
    pub description: Option<Node<str>>,
    pub name: Name,
    pub directives: DirectiveList,
    pub values: IndexMap<Name, Node<EnumValueDefinition>>,
}

/// The definition of an [input object type](https://spec.graphql.org/September2025/#sec-Input-Objects),
/// with all information from type extensions folded in.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InputObjectType {
    pub description: Option<Node<str>>,
    pub name: Name,
    pub directives: DirectiveList,
    pub fields: IndexMap<Name, Node<InputValueDefinition>>,
}

/// The names of all types that implement a given interface.
/// Returned by [`Schema::implementers_map`].
///
/// Concrete object types and derived interfaces can be accessed separately.
///
/// # Examples
///
/// ```rust
/// use apollo_compiler::schema::Implementers;
/// # let implementers = Implementers::default();
///
/// // introspection must return only concrete implementers.
/// let possible_types = implementers.objects;
/// ```
///
/// ```rust
/// use apollo_compiler::schema::Implementers;
/// # let implementers = Implementers::default();
///
/// for name in implementers.iter() {
///     // iterates both concrete objects and interfaces
///     println!("{name}");
/// }
/// ```
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Implementers {
    /// Names of the concrete object types that implement an interface.
    pub objects: IndexSet<Name>,
    /// Names of the interface types that implement an interface.
    pub interfaces: IndexSet<Name>,
}

/// AST node that has been skipped during conversion to `Schema`
#[derive(thiserror::Error, Debug, Clone)]
pub(crate) enum BuildError {
    #[error("a schema document must not contain {describe}")]
    ExecutableDefinition { describe: &'static str },

    #[error("must not have multiple `schema` definitions")]
    SchemaDefinitionCollision {
        previous_location: Option<SourceSpan>,
    },

    #[error("the directive `@{name}` is defined multiple times in the schema")]
    DirectiveDefinitionCollision {
        previous_location: Option<SourceSpan>,
        name: Name,
    },

    #[error("the type `{name}` is defined multiple times in the schema")]
    TypeDefinitionCollision {
        previous_location: Option<SourceSpan>,
        name: Name,
    },

    #[error("built-in scalar definitions must be omitted")]
    BuiltInScalarTypeRedefinition,

    #[error("schema extension without a schema definition")]
    OrphanSchemaExtension,

    #[error("type extension for undefined type `{name}`")]
    OrphanTypeExtension { name: Name },

    #[error("adding {describe_ext}, but `{name}` is {describe_def}")]
    TypeExtensionKindMismatch {
        name: Name,
        describe_ext: &'static str,
        def_location: Option<SourceSpan>,
        describe_def: &'static str,
    },

    #[error("duplicate definitions for the `{operation_type}` root operation type")]
    DuplicateRootOperation {
        previous_location: Option<SourceSpan>,
        operation_type: &'static str,
    },

    #[error(
        "object type `{type_name}` implements interface `{name_at_previous_location}` \
         more than once"
    )]
    DuplicateImplementsInterfaceInObject {
        name_at_previous_location: Name,
        type_name: Name,
    },

    #[error(
        "interface type `{type_name}` implements interface `{name_at_previous_location}` \
         more than once"
    )]
    DuplicateImplementsInterfaceInInterface {
        name_at_previous_location: Name,
        type_name: Name,
    },

    #[error(
        "duplicate definitions for the `{name_at_previous_location}` \
         field of object type `{type_name}`"
    )]
    ObjectFieldNameCollision {
        name_at_previous_location: Name,
        type_name: Name,
    },

    #[error(
        "duplicate definitions for the `{name_at_previous_location}` \
         field of interface type `{type_name}`"
    )]
    InterfaceFieldNameCollision {
        name_at_previous_location: Name,
        type_name: Name,
    },

    #[error(
        "duplicate definitions for the `{name_at_previous_location}` \
         value of enum type `{type_name}`"
    )]
    EnumValueNameCollision {
        name_at_previous_location: Name,
        type_name: Name,
    },

    #[error(
        "duplicate definitions for the `{name_at_previous_location}` \
         member of union type `{type_name}`"
    )]
    UnionMemberNameCollision {
        name_at_previous_location: Name,
        type_name: Name,
    },

    #[error(
        "duplicate definitions for the `{name_at_previous_location}` \
         field of input object type `{type_name}`"
    )]
    InputFieldNameCollision {
        name_at_previous_location: Name,
        type_name: Name,
    },
}

/// Error type of [`Schema::type_field`]: could not find the requested field definition
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FieldLookupError<'schema> {
    NoSuchType,
    NoSuchField(&'schema NamedType, &'schema ExtendedType),
}

impl Schema {
    /// Returns an (almost) empty schema.
    ///
    /// It starts with built-in directives, built-in scalars, and introspection types.
    /// It can then be filled programatically.
    #[allow(clippy::new_without_default)] // not a great implicit default in generic contexts
    pub fn new() -> Self {
        SchemaBuilder::new().build().unwrap()
    }

    /// Parse a single source file into a schema, with the default parser configuration.
    ///
    /// Create a [`Parser`] to use different parser configuration.
    /// Use [`builder()`][Self::builder] to build a schema from multiple parsed files.
    #[allow(clippy::result_large_err)] // Typically not called very often
    pub fn parse(
        source_text: impl Into<String>,
        path: impl AsRef<Path>,
    ) -> Result<Self, WithErrors<Self>> {
        Parser::default().parse_schema(source_text, path)
    }

    /// [`parse`][Self::parse] then [`validate`][Self::validate],
    /// to get a `Valid<Schema>` when mutating it isn’t needed.
    #[allow(clippy::result_large_err)] // Typically not called very often
    pub fn parse_and_validate(
        source_text: impl Into<String>,
        path: impl AsRef<Path>,
    ) -> Result<Valid<Self>, WithErrors<Self>> {
        let mut builder = Schema::builder();
        Parser::default().parse_into_schema_builder(source_text, path, &mut builder);
        let (mut schema, mut errors) = builder.build_inner();
        validation::validate_schema(&mut errors, &mut schema);
        errors.into_valid_result(schema)
    }

    /// Returns a new builder for creating a Schema from AST documents,
    /// initialized with built-in directives, built-in scalars, and introspection types
    ///
    /// ```rust
    /// use apollo_compiler::Schema;
    ///
    /// let empty_schema = Schema::builder().build();
    /// ```
    pub fn builder() -> SchemaBuilder {
        SchemaBuilder::new()
    }

    #[allow(clippy::result_large_err)] // Typically not called very often
    pub fn validate(mut self) -> Result<Valid<Self>, WithErrors<Self>> {
        let mut errors = DiagnosticList::new(self.sources.clone());
        validation::validate_schema(&mut errors, &mut self);
        errors.into_valid_result(self)
    }

    /// Returns the type with the given name, if it is a scalar type
    pub fn get_scalar(&self, name: &str) -> Option<&Node<ScalarType>> {
        if let Some(ExtendedType::Scalar(ty)) = self.types.get(name) {
            Some(ty)
        } else {
            None
        }
    }

    /// Returns the type with the given name, if it is a object type
    pub fn get_object(&self, name: &str) -> Option<&Node<ObjectType>> {
        if let Some(ExtendedType::Object(ty)) = self.types.get(name) {
            Some(ty)
        } else {
            None
        }
    }

    /// Returns the type with the given name, if it is a interface type
    pub fn get_interface(&self, name: &str) -> Option<&Node<InterfaceType>> {
        if let Some(ExtendedType::Interface(ty)) = self.types.get(name) {
            Some(ty)
        } else {
            None
        }
    }

    /// Returns the type with the given name, if it is a union type
    pub fn get_union(&self, name: &str) -> Option<&Node<UnionType>> {
        if let Some(ExtendedType::Union(ty)) = self.types.get(name) {
            Some(ty)
        } else {
            None
        }
    }

    /// Returns the type with the given name, if it is a enum type
    pub fn get_enum(&self, name: &str) -> Option<&Node<EnumType>> {
        if let Some(ExtendedType::Enum(ty)) = self.types.get(name) {
            Some(ty)
        } else {
            None
        }
    }

    /// Returns the type with the given name, if it is a input object type
    pub fn get_input_object(&self, name: &str) -> Option<&Node<InputObjectType>> {
        if let Some(ExtendedType::InputObject(ty)) = self.types.get(name) {
            Some(ty)
        } else {
            None
        }
    }

    /// Returns the name of the object type for the root operation with the given operation kind
    pub fn root_operation(&self, operation_type: ast::OperationType) -> Option<&NamedType> {
        match operation_type {
            ast::OperationType::Query => &self.schema_definition.query,
            ast::OperationType::Mutation => &self.schema_definition.mutation,
            ast::OperationType::Subscription => &self.schema_definition.subscription,
        }
        .as_ref()
        .map(|component| component.as_ref())
    }

    /// Returns the definition of a type’s explicit field or meta-field.
    pub fn type_field(
        &self,
        type_name: &str,
        field_name: &str,
    ) -> Result<&Node<FieldDefinition>, FieldLookupError<'_>> {
        use ExtendedType::*;
        let (ty_def_name, ty_def) = self
            .types
            .get_key_value(type_name)
            .ok_or(FieldLookupError::NoSuchType)?;
        let explicit_field = match ty_def {
            Object(ty) => ty.fields.get(field_name),
            Interface(ty) => ty.fields.get(field_name),
            Scalar(_) | Union(_) | Enum(_) | InputObject(_) => None,
        };
        if let Some(def) = explicit_field {
            return Ok(def);
        }
        let meta = MetaFieldDefinitions::get();
        if field_name == "__typename" && matches!(ty_def, Object(_) | Interface(_) | Union(_)) {
            // .validate() errors for __typename at the root of a subscription operation
            return Ok(&meta.__typename);
        }
        if self
            .schema_definition
            .query
            .as_ref()
            .is_some_and(|query_type| query_type == type_name)
        {
            match field_name {
                "__schema" => return Ok(&meta.__schema),
                "__type" => return Ok(&meta.__type),
                _ => {}
            }
        }
        Err(FieldLookupError::NoSuchField(ty_def_name, ty_def))
    }

    /// Returns a map of interface names to names of types that implement that interface
    ///
    /// `Schema` only stores the inverse relationship
    /// (in [`ObjectType::implements_interfaces`] and [`InterfaceType::implements_interfaces`]),
    /// so iterating the implementers of an interface requires a linear scan
    /// of all types in the schema.
    /// If that is repeated for multiple interfaces,
    /// gathering them all at once amorticizes that cost.
    pub fn implementers_map(&self) -> HashMap<Name, Implementers> {
        let mut map = HashMap::<Name, Implementers>::default();
        for (ty_name, ty) in &self.types {
            match ty {
                ExtendedType::Object(def) => {
                    for interface in &def.implements_interfaces {
                        map.entry(interface.as_ref().clone())
                            .or_default()
                            .objects
                            .insert(ty_name.clone());
                    }
                }
                ExtendedType::Interface(def) => {
                    for interface in &def.implements_interfaces {
                        map.entry(interface.as_ref().clone())
                            .or_default()
                            .interfaces
                            .insert(ty_name.clone());
                    }
                }
                ExtendedType::Scalar(_)
                | ExtendedType::Union(_)
                | ExtendedType::Enum(_)
                | ExtendedType::InputObject(_) => (),
            };
        }
        map
    }

    /// Returns whether `maybe_subtype` is a subtype of `abstract_type`, which means either:
    ///
    /// * `maybe_subtype` implements the interface `abstract_type`
    /// * `maybe_subtype` is a member of the union type `abstract_type`
    pub fn is_subtype(&self, abstract_type: &str, maybe_subtype: &str) -> bool {
        self.types.get(abstract_type).is_some_and(|ty| match ty {
            ExtendedType::Interface(_) => self.types.get(maybe_subtype).is_some_and(|ty2| {
                match ty2 {
                    ExtendedType::Object(def) => &def.implements_interfaces,
                    ExtendedType::Interface(def) => &def.implements_interfaces,
                    ExtendedType::Scalar(_)
                    | ExtendedType::Union(_)
                    | ExtendedType::Enum(_)
                    | ExtendedType::InputObject(_) => return false,
                }
                .contains(abstract_type)
            }),
            ExtendedType::Union(def) => def.members.contains(maybe_subtype),
            ExtendedType::Scalar(_)
            | ExtendedType::Object(_)
            | ExtendedType::Enum(_)
            | ExtendedType::InputObject(_) => false,
        })
    }

    /// Returns whether the type `ty` is defined as is an input type
    ///
    /// <https://spec.graphql.org/September2025/#sec-Input-and-Output-Types>
    pub fn is_input_type(&self, ty: &Type) -> bool {
        match self.types.get(ty.inner_named_type()) {
            Some(ExtendedType::Scalar(_))
            | Some(ExtendedType::Enum(_))
            | Some(ExtendedType::InputObject(_)) => true,
            Some(ExtendedType::Object(_))
            | Some(ExtendedType::Interface(_))
            | Some(ExtendedType::Union(_))
            | None => false,
        }
    }

    /// Returns whether the type `ty` is defined as is an output type
    ///
    /// <https://spec.graphql.org/September2025/#sec-Input-and-Output-Types>
    pub fn is_output_type(&self, ty: &Type) -> bool {
        match self.types.get(ty.inner_named_type()) {
            Some(ExtendedType::Scalar(_))
            | Some(ExtendedType::Object(_))
            | Some(ExtendedType::Interface(_))
            | Some(ExtendedType::Union(_))
            | Some(ExtendedType::Enum(_)) => true,
            Some(ExtendedType::InputObject(_)) | None => false,
        }
    }

    serialize_method!();
}

impl SchemaDefinition {
    pub fn iter_root_operations(&self) -> impl Iterator<Item = (ast::OperationType, &Node<Name>)> {
        [
            (ast::OperationType::Query, &self.query),
            (ast::OperationType::Mutation, &self.mutation),
            (ast::OperationType::Subscription, &self.subscription),
        ]
        .into_iter()
        .filter_map(|(ty, maybe_op)| maybe_op.as_ref().map(|op| (ty, op)))
    }

    /// Iterate over the `origins` of all components
    ///
    /// The order of the returned set is unspecified but deterministic
    /// for a given apollo-compiler version.
    pub fn iter_extension_ids(&self) -> impl Iterator<Item = Option<&ExtensionId>> {
        self.directives
            .iter()
            .map(|dir| dir.extension_id())
            .chain(self.query.iter().map(|name| name.extension_id()))
            .chain(self.mutation.iter().map(|name| name.extension_id()))
            .chain(self.subscription.iter().map(|name| name.extension_id()))
    }

    /// Collect `schema` extensions that contribute any component
    ///
    /// The order of the returned set is unspecified but deterministic
    /// for a given apollo-compiler version.
    pub fn extensions(&self) -> IndexSet<&ExtensionId> {
        self.iter_extension_ids()
            .flatten()
            .collect()
    }
}

impl ExtendedType {
    pub fn name(&self) -> &Name {
        match self {
            Self::Scalar(def) => &def.name,
            Self::Object(def) => &def.name,
            Self::Interface(def) => &def.name,
            Self::Union(def) => &def.name,
            Self::Enum(def) => &def.name,
            Self::InputObject(def) => &def.name,
        }
    }

    /// Return the source location of the type's base definition.
    ///
    /// If the type has extensions, those are not covered by this location.
    pub fn location(&self) -> Option<SourceSpan> {
        match self {
            Self::Scalar(ty) => ty.location(),
            Self::Object(ty) => ty.location(),
            Self::Interface(ty) => ty.location(),
            Self::Union(ty) => ty.location(),
            Self::Enum(ty) => ty.location(),
            Self::InputObject(ty) => ty.location(),
        }
    }

    pub(crate) fn describe(&self) -> &'static str {
        match self {
            Self::Scalar(_) => "a scalar type",
            Self::Object(_) => "an object type",
            Self::Interface(_) => "an interface type",
            Self::Union(_) => "a union type",
            Self::Enum(_) => "an enum type",
            Self::InputObject(_) => "an input object type",
        }
    }

    pub fn is_scalar(&self) -> bool {
        matches!(self, Self::Scalar(_))
    }

    pub fn is_object(&self) -> bool {
        matches!(self, Self::Object(_))
    }

    pub fn is_interface(&self) -> bool {
        matches!(self, Self::Interface(_))
    }

    pub fn is_union(&self) -> bool {
        matches!(self, Self::Union(_))
    }

    pub fn is_enum(&self) -> bool {
        matches!(self, Self::Enum(_))
    }

    pub fn is_input_object(&self) -> bool {
        matches!(self, Self::InputObject(_))
    }

    pub fn as_scalar(&self) -> Option<&ScalarType> {
        if let Self::Scalar(def) = self {
            Some(def)
        } else {
            None
        }
    }

    pub fn as_object(&self) -> Option<&ObjectType> {
        if let Self::Object(def) = self {
            Some(def)
        } else {
            None
        }
    }

    pub fn as_interface(&self) -> Option<&InterfaceType> {
        if let Self::Interface(def) = self {
            Some(def)
        } else {
            None
        }
    }

    pub fn as_union(&self) -> Option<&UnionType> {
        if let Self::Union(def) = self {
            Some(def)
        } else {
            None
        }
    }

    pub fn as_enum(&self) -> Option<&EnumType> {
        if let Self::Enum(def) = self {
            Some(def)
        } else {
            None
        }
    }

    pub fn as_input_object(&self) -> Option<&InputObjectType> {
        if let Self::InputObject(def) = self {
            Some(def)
        } else {
            None
        }
    }

    /// Returns wether this type is a leaf type: scalar or enum.
    ///
    /// Field selections must have sub-selections if and only if
    /// their inner named type is *not* a leaf field.
    pub fn is_leaf(&self) -> bool {
        matches!(self, Self::Scalar(_) | Self::Enum(_))
    }

    /// Returns true if a value of this type can be used as an input value.
    ///
    /// # Spec
    /// This implements spec function
    /// [`IsInputType(type)`](https://spec.graphql.org/September2025/#IsInputType())
    pub fn is_input_type(&self) -> bool {
        matches!(self, Self::Scalar(_) | Self::Enum(_) | Self::InputObject(_))
    }

    /// Returns true if a value of this type can be used as an output value.
    ///
    /// # Spec
    /// This implements spec function
    /// [`IsOutputType(type)`](https://spec.graphql.org/September2025/#IsOutputType())
    pub fn is_output_type(&self) -> bool {
        matches!(
            self,
            Self::Scalar(_) | Self::Enum(_) | Self::Object(_) | Self::Interface(_) | Self::Union(_)
        )
    }

    /// Returns whether this is a built-in scalar or introspection type
    pub fn is_built_in(&self) -> bool {
        match self {
            Self::Scalar(ty) => ty.is_built_in(),
            Self::Object(ty) => ty.is_built_in(),
            Self::Interface(ty) => ty.is_built_in(),
            Self::Union(ty) => ty.is_built_in(),
            Self::Enum(ty) => ty.is_built_in(),
            Self::InputObject(ty) => ty.is_built_in(),
        }
    }

    pub fn directives(&self) -> &DirectiveList {
        match self {
            Self::Scalar(ty) => &ty.directives,
            Self::Object(ty) => &ty.directives,
            Self::Interface(ty) => &ty.directives,
            Self::Union(ty) => &ty.directives,
            Self::Enum(ty) => &ty.directives,
            Self::InputObject(ty) => &ty.directives,
        }
    }

    pub fn description(&self) -> Option<&Node<str>> {
        match self {
            Self::Scalar(ty) => ty.description.as_ref(),
            Self::Object(ty) => ty.description.as_ref(),
            Self::Interface(ty) => ty.description.as_ref(),
            Self::Union(ty) => ty.description.as_ref(),
            Self::Enum(ty) => ty.description.as_ref(),
            Self::InputObject(ty) => ty.description.as_ref(),
        }
    }

    /// Iterate over the `origins` of all components
    ///
    /// The order of the returned set is unspecified but deterministic
    /// for a given apollo-compiler version.
    pub fn iter_extension_ids(&self) -> impl Iterator<Item = Option<&ExtensionId>> {
        match self {
            Self::Scalar(ty) => Box::new(ty.iter_extension_ids()) as Box<dyn Iterator<Item = _>>,
            Self::Object(ty) => Box::new(ty.iter_extension_ids()),
            Self::Interface(ty) => Box::new(ty.iter_extension_ids()),
            Self::Union(ty) => Box::new(ty.iter_extension_ids()),
            Self::Enum(ty) => Box::new(ty.iter_extension_ids()),
            Self::InputObject(ty) => Box::new(ty.iter_extension_ids()),
        }
    }

    /// Collect `schema` extensions that contribute any component
    ///
    /// The order of the returned set is unspecified but deterministic
    /// for a given apollo-compiler version.
    pub fn extensions(&self) -> IndexSet<&ExtensionId> {
        self.iter_extension_ids()
            .flatten()
            .collect()
    }

    serialize_method!();
}

impl ScalarType {
    /// Iterate over the `origins` of all components
    ///
    /// The order of the returned set is unspecified but deterministic
    /// for a given apollo-compiler version.
    pub fn iter_extension_ids(&self) -> impl Iterator<Item = Option<&ExtensionId>> {
        self.directives.iter().map(|dir| dir.extension_id())
    }

    /// Collect scalar type extensions that contribute any component
    ///
    /// The order of the returned set is unspecified but deterministic
    /// for a given apollo-compiler version.
    pub fn extensions(&self) -> IndexSet<&ExtensionId> {
        self.iter_extension_ids()
            .flatten()
            .collect()
    }

    serialize_method!();
}

impl ObjectType {
    /// Iterate over the `origins` of all components
    ///
    /// The order of the returned set is unspecified but deterministic
    /// for a given apollo-compiler version.
    pub fn iter_extension_ids(&self) -> impl Iterator<Item = Option<&ExtensionId>> {
        self.directives
            .iter()
            .map(|dir| dir.extension_id())
            .chain(
                self.implements_interfaces
                    .iter()
                    .map(|component| component.extension_id()),
            )
            .chain(self.fields.values().map(|field| field.extension_id()))
    }

    /// Collect object type extensions that contribute any component
    ///
    /// The order of the returned set is unspecified but deterministic
    /// for a given apollo-compiler version.
    pub fn extensions(&self) -> IndexSet<&ExtensionId> {
        self.iter_extension_ids()
            .flatten()
            .collect()
    }

    serialize_method!();
}

impl InterfaceType {
    /// Iterate over the `origins` of all components
    ///
    /// The order of the returned set is unspecified but deterministic
    /// for a given apollo-compiler version.
    pub fn iter_extension_ids(&self) -> impl Iterator<Item = Option<&ExtensionId>> {
        self.directives
            .iter()
            .map(|dir| dir.extension_id())
            .chain(
                self.implements_interfaces
                    .iter()
                    .map(|component| component.extension_id()),
            )
            .chain(self.fields.values().map(|field| field.extension_id()))
    }

    /// Collect interface type extensions that contribute any component
    ///
    /// The order of the returned set is unspecified but deterministic
    /// for a given apollo-compiler version.
    pub fn extensions(&self) -> IndexSet<&ExtensionId> {
        self.iter_extension_ids()
            .flatten()
            .collect()
    }

    serialize_method!();
}

impl UnionType {
    /// Iterate over the `origins` of all components
    ///
    /// The order of the returned set is unspecified but deterministic
    /// for a given apollo-compiler version.
    pub fn iter_extension_ids(&self) -> impl Iterator<Item = Option<&ExtensionId>> {
        self.directives
            .iter()
            .map(|dir| dir.extension_id())
            .chain(self.members.iter().map(|component| component.extension_id()))
    }

    /// Collect union type extensions that contribute any component
    ///
    /// The order of the returned set is unspecified but deterministic
    /// for a given apollo-compiler version.
    pub fn extensions(&self) -> IndexSet<&ExtensionId> {
        self.iter_extension_ids()
            .flatten()
            .collect()
    }

    serialize_method!();
}

impl EnumType {
    /// Iterate over the `origins` of all components
    ///
    /// The order of the returned set is unspecified but deterministic
    /// for a given apollo-compiler version.
    pub fn iter_extension_ids(&self) -> impl Iterator<Item = Option<&ExtensionId>> {
        self.directives
            .iter()
            .map(|dir| dir.extension_id())
            .chain(self.values.values().map(|value| value.extension_id()))
    }

    /// Collect enum type extensions that contribute any component
    ///
    /// The order of the returned set is unspecified but deterministic
    /// for a given apollo-compiler version.
    pub fn extensions(&self) -> IndexSet<&ExtensionId> {
        self.iter_extension_ids()
            .flatten()
            .collect()
    }

    serialize_method!();
}

impl InputObjectType {
    /// Returns true if this is a OneOf Input Object (has the `@oneOf` directive).
    pub fn is_one_of(&self) -> bool {
        self.directives.get("oneOf").is_some()
    }

    /// Iterate over the `origins` of all components
    ///
    /// The order of the returned set is unspecified but deterministic
    /// for a given apollo-compiler version.
    pub fn iter_extension_ids(&self) -> impl Iterator<Item = Option<&ExtensionId>> {
        self.directives
            .iter()
            .map(|dir| dir.extension_id())
            .chain(self.fields.values().map(|field| field.extension_id()))
    }

    /// Collect input object type extensions that contribute any component
    ///
    /// The order of the returned set is unspecified but deterministic
    /// for a given apollo-compiler version.
    pub fn extensions(&self) -> IndexSet<&ExtensionId> {
        self.iter_extension_ids()
            .flatten()
            .collect()
    }

    serialize_method!();
}

impl Eq for Schema {}

impl PartialEq for Schema {
    fn eq(&self, other: &Self) -> bool {
        let Self {
            sources: _,                 // ignored
            validate_default_values: _, // ignored, config only
            schema_definition,
            directive_definitions,
            types,
        } = self;
        *schema_definition == other.schema_definition
            && *directive_definitions == other.directive_definitions
            && *types == other.types
    }
}

impl Implementers {
    /// Iterate over all implementers, including objects and interfaces.
    ///
    /// The iteration order is unspecified.
    pub fn iter(&self) -> impl Iterator<Item = &'_ Name> {
        self.objects.iter().chain(&self.interfaces)
    }
}

impl From<Node<ScalarType>> for ExtendedType {
    fn from(ty: Node<ScalarType>) -> Self {
        Self::Scalar(ty)
    }
}

impl From<Node<ObjectType>> for ExtendedType {
    fn from(ty: Node<ObjectType>) -> Self {
        Self::Object(ty)
    }
}

impl From<Node<InterfaceType>> for ExtendedType {
    fn from(ty: Node<InterfaceType>) -> Self {
        Self::Interface(ty)
    }
}

impl From<Node<UnionType>> for ExtendedType {
    fn from(ty: Node<UnionType>) -> Self {
        Self::Union(ty)
    }
}

impl From<Node<EnumType>> for ExtendedType {
    fn from(ty: Node<EnumType>) -> Self {
        Self::Enum(ty)
    }
}

impl From<Node<InputObjectType>> for ExtendedType {
    fn from(ty: Node<InputObjectType>) -> Self {
        Self::InputObject(ty)
    }
}

impl From<ScalarType> for ExtendedType {
    fn from(ty: ScalarType) -> Self {
        Self::Scalar(ty.into())
    }
}

impl From<ObjectType> for ExtendedType {
    fn from(ty: ObjectType) -> Self {
        Self::Object(ty.into())
    }
}

impl From<InterfaceType> for ExtendedType {
    fn from(ty: InterfaceType) -> Self {
        Self::Interface(ty.into())
    }
}

impl From<UnionType> for ExtendedType {
    fn from(ty: UnionType) -> Self {
        Self::Union(ty.into())
    }
}

impl From<EnumType> for ExtendedType {
    fn from(ty: EnumType) -> Self {
        Self::Enum(ty.into())
    }
}

impl From<InputObjectType> for ExtendedType {
    fn from(ty: InputObjectType) -> Self {
        Self::InputObject(ty.into())
    }
}

impl std::fmt::Debug for Schema {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self {
            sources,
            schema_definition,
            directive_definitions,
            types,
            validate_default_values: _,
        } = self;
        f.debug_struct("Schema")
            .field("sources", sources)
            .field("schema_definition", schema_definition)
            .field(
                "directive_definitions",
                &DebugDirectiveDefinitions(directive_definitions),
            )
            .field("types", &DebugTypes(types))
            .finish()
    }
}

struct DebugDirectiveDefinitions<'a>(&'a IndexMap<Name, Node<DirectiveDefinition>>);

struct DebugTypes<'a>(&'a IndexMap<Name, ExtendedType>);

impl std::fmt::Debug for DebugDirectiveDefinitions<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut map = f.debug_map();
        for (name, def) in self.0 {
            if !def.is_built_in() {
                map.entry(name, def);
            } else {
                map.entry(name, &format_args!("built_in_directive!({name:?})"));
            }
        }
        map.finish()
    }
}

impl std::fmt::Debug for DebugTypes<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut map = f.debug_map();
        for (name, def) in self.0 {
            if !def.is_built_in() {
                map.entry(name, def);
            } else {
                map.entry(name, &format_args!("built_in_type!({name:?})"));
            }
        }
        map.finish()
    }
}

struct MetaFieldDefinitions {
    __typename: Node<FieldDefinition>,
    __schema: Node<FieldDefinition>,
    __type: Node<FieldDefinition>,
}

impl MetaFieldDefinitions {
    fn get() -> &'static Self {
        static DEFS: OnceLock<MetaFieldDefinitions> = OnceLock::new();
        DEFS.get_or_init(|| Self {
            // __typename: String!
            __typename: Node::new(FieldDefinition {
                description: None,
                name: name!("__typename"),
                arguments: Vec::new(),
                ty: ty!(String!),
                directives: ast::DirectiveList::new(),
            }),
            // __schema: __Schema!
            __schema: Node::new(FieldDefinition {
                description: None,
                name: name!("__schema"),
                arguments: Vec::new(),
                ty: ty!(__Schema!),
                directives: ast::DirectiveList::new(),
            }),
            // __type(name: String!): __Type
            __type: Node::new(FieldDefinition {
                description: None,
                name: name!("__type"),
                arguments: vec![InputValueDefinition {
                    description: None,
                    name: name!("name"),
                    ty: ty!(String!).into(),
                    default_value: None,
                    directives: ast::DirectiveList::new(),
                }
                .into()],
                ty: ty!(__Type),
                directives: ast::DirectiveList::new(),
            }),
        })
    }
}
