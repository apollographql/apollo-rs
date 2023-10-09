//! High-level representation of a GraphQL schema

use crate::ast;
use crate::FileId;
use crate::Node;
use crate::NodeLocation;
use crate::NodeStr;
use crate::Parser;
use indexmap::IndexMap;
use indexmap::IndexSet;
use std::collections::HashMap;
use std::collections::HashSet;
use std::path::Path;
use std::sync::OnceLock;

mod component;
mod from_ast;
mod serialize;
mod validation;

pub use self::component::{Component, ComponentOrigin, ComponentStr, ExtensionId};
pub use self::from_ast::SchemaBuilder;
pub use crate::ast::{
    Directive, DirectiveDefinition, DirectiveLocation, EnumValueDefinition, FieldDefinition,
    InputValueDefinition, Name, NamedType, Type, Value,
};
use crate::validation::Diagnostics;

/// High-level representation of a GraphQL schema
#[derive(Clone)]
pub struct Schema {
    /// Source files, if any, that were parsed to contribute to this schema.
    ///
    /// The schema (including parsed definitions) may have been modified since parsing.
    pub sources: crate::SourceMap,

    /// Errors that occurred when building this schema,
    /// either parsing a source file or converting from AST.
    build_errors: Vec<BuildError>,

    /// The `schema` definition and its extensions, defining root operations
    pub schema_definition: Node<SchemaDefinition>,

    /// Built-in and explicit directive definitions
    pub directive_definitions: IndexMap<Name, Node<DirectiveDefinition>>,

    /// Definitions and extensions of built-in scalars, introspection types,
    /// and explicit types
    pub types: IndexMap<NamedType, ExtendedType>,
}

/// The `schema` definition and its extensions, defining root operations
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SchemaDefinition {
    pub description: Option<NodeStr>,
    pub directives: Directives,

    /// Name of the object type for the `query` root operation
    pub query: Option<ComponentStr>,

    /// Name of the object type for the `mutation` root operation
    pub mutation: Option<ComponentStr>,

    /// Name of the object type for the `subscription` root operation
    pub subscription: Option<ComponentStr>,
}

#[derive(Clone, Eq, PartialEq, Hash, Default)]
pub struct Directives(pub Vec<Component<Directive>>);

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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ScalarType {
    pub description: Option<NodeStr>,
    pub directives: Directives,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObjectType {
    pub description: Option<NodeStr>,
    pub implements_interfaces: IndexSet<ComponentStr>,
    pub directives: Directives,

    /// Explicit field definitions.
    ///
    /// When looking up a definition,
    /// consider using [`Schema::type_field`] instead to include meta-fields.
    pub fields: IndexMap<Name, Component<FieldDefinition>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InterfaceType {
    pub description: Option<NodeStr>,

    /// * Key: name of an implemented interface
    /// * Value: which interface type extension defined this implementation,
    ///   or `None` for the interface type definition.
    pub implements_interfaces: IndexSet<ComponentStr>,

    pub directives: Directives,

    /// Explicit field definitions.
    ///
    /// When looking up a definition,
    /// consider using [`Schema::type_field`] instead to include meta-fields.
    pub fields: IndexMap<Name, Component<FieldDefinition>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnionType {
    pub description: Option<NodeStr>,
    pub directives: Directives,

    /// * Key: name of a member object type
    /// * Value: which union type extension defined this implementation,
    ///   or `None` for the union type definition.
    pub members: IndexSet<ComponentStr>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnumType {
    pub description: Option<NodeStr>,
    pub directives: Directives,
    pub values: IndexMap<Name, Component<EnumValueDefinition>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InputObjectType {
    pub description: Option<NodeStr>,
    pub directives: Directives,
    pub fields: IndexMap<Name, Component<InputValueDefinition>>,
}

/// AST node that has been skipped during conversion to `Schema`
#[derive(thiserror::Error, Debug, Clone)]
pub(crate) enum BuildError {
    #[error("a schema document must not contain {describe}")]
    ExecutableDefinition {
        location: Option<NodeLocation>,
        describe: &'static str,
    },

    #[error("must not have multiple `schema` definitions")]
    SchemaDefinitionCollision {
        location: Option<NodeLocation>,
        previous_location: Option<NodeLocation>,
    },

    #[error("the directive `@{name}` is defined multiple times in the schema")]
    DirectiveDefinitionCollision {
        location: Option<NodeLocation>,
        previous_location: Option<NodeLocation>,
        name: Name,
    },

    #[error("the type `{name}` is defined multiple times in the schema")]
    TypeDefinitionCollision {
        location: Option<NodeLocation>,
        previous_location: Option<NodeLocation>,
        name: Name,
    },

    #[error("built-in scalar definitions must be omitted")]
    BuiltInScalarTypeRedefinition { location: Option<NodeLocation> },

    #[error("schema extension without a schema definition")]
    OrphanSchemaExtension { location: Option<NodeLocation> },

    #[error("type extension for undefined type `{name}`")]
    OrphanTypeExtension {
        location: Option<NodeLocation>,
        name: Name,
    },

    #[error("adding {describe_ext}, but `{name}` is {describe_def}")]
    TypeExtensionKindMismatch {
        location: Option<NodeLocation>,
        name: Name,
        describe_ext: &'static str,
        def_location: Option<NodeLocation>,
        describe_def: &'static str,
    },

    #[error("duplicate definitions for the `{operation_type}` root operation type")]
    DuplicateRootOperation {
        location: Option<NodeLocation>,
        previous_location: Option<NodeLocation>,
        operation_type: &'static str,
    },

    #[error(
        "object type `{type_name}` implements interface `{name_at_previous_location}` \
         more than once"
    )]
    DuplicateImplementsInterfaceInObject {
        location: Option<NodeLocation>,
        name_at_previous_location: Name,
        type_name: Name,
    },

    #[error(
        "interface type `{type_name}` implements interface `{name_at_previous_location}` \
         more than once"
    )]
    DuplicateImplementsInterfaceInInterface {
        location: Option<NodeLocation>,
        name_at_previous_location: Name,
        type_name: Name,
    },

    #[error(
        "duplicate definitions for the `{name_at_previous_location}` \
         field of object type `{type_name}`"
    )]
    ObjectFieldNameCollision {
        location: Option<NodeLocation>,
        name_at_previous_location: Name,
        type_name: Name,
    },

    #[error(
        "duplicate definitions for the `{name_at_previous_location}` \
         field of interface type `{type_name}`"
    )]
    InterfaceFieldNameCollision {
        location: Option<NodeLocation>,
        name_at_previous_location: Name,
        type_name: Name,
    },

    #[error(
        "duplicate definitions for the `{name_at_previous_location}` \
         value of enum type `{type_name}`"
    )]
    EnumValueNameCollision {
        location: Option<NodeLocation>,
        name_at_previous_location: Name,
        type_name: Name,
    },

    #[error(
        "duplicate definitions for the `{name_at_previous_location}` \
         member of union type `{type_name}`"
    )]
    UnionMemberNameCollision {
        location: Option<NodeLocation>,
        name_at_previous_location: Name,
        type_name: Name,
    },

    #[error(
        "duplicate definitions for the `{name_at_previous_location}` \
         field of input object type `{type_name}`"
    )]
    InputFieldNameCollision {
        location: Option<NodeLocation>,
        name_at_previous_location: Name,
        type_name: Name,
    },
}

/// Could not find the requested field definition
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
        SchemaBuilder::new().build()
    }

    /// Parse a single source file into a schema, with the default parser configuration.
    ///
    /// Create a [`Parser`] to use different parser configuration.
    /// Use [`builder()`][Self::builder] to build a schema from multiple parsed files.
    pub fn parse(source_text: impl Into<String>, path: impl AsRef<Path>) -> Self {
        Parser::default().parse_schema(source_text, path)
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

    /// Returns `Err` if invalid, or `Ok` for potential warnings or advice
    pub fn validate(&self) -> Result<Diagnostics, Diagnostics> {
        let mut errors = Diagnostics::new(None, self.sources.clone());
        let warnings_and_advice = validation::validate_schema(&mut errors, self);
        let valid = errors.is_empty();
        for diagnostic in warnings_and_advice {
            errors.push(
                Some(diagnostic.location),
                crate::validation::Details::CompilerDiagnostic(diagnostic),
            )
        }
        errors.sort();
        if valid {
            Ok(errors)
        } else {
            Err(errors)
        }
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
        .map(|component| &component.node)
    }

    /// Returns the definition of a type’s explicit field or meta-field.
    pub fn type_field(
        &self,
        type_name: &str,
        field_name: &str,
    ) -> Result<&Component<FieldDefinition>, FieldLookupError<'_>> {
        let (ty_def_name, ty_def) = self
            .types
            .get_key_value(type_name)
            .ok_or(FieldLookupError::NoSuchType)?;
        self.meta_fields_definitions(type_name)
            .iter()
            .find(|def| def.name == field_name)
            .or_else(|| match ty_def {
                ExtendedType::Object(ty) => ty.fields.get(field_name),
                ExtendedType::Interface(ty) => ty.fields.get(field_name),
                ExtendedType::Scalar(_)
                | ExtendedType::Union(_)
                | ExtendedType::Enum(_)
                | ExtendedType::InputObject(_) => None,
            })
            .ok_or(FieldLookupError::NoSuchField(ty_def_name, ty_def))
    }

    /// Returns a map of interface names to names of types that implement that interface
    ///
    /// `Schema` only stores the inverse relationship
    /// (in [`ObjectType::implements_interfaces`] and [`InterfaceType::implements_interfaces`]),
    /// so iterating the implementers of an interface requires a linear scan
    /// of all types in the schema.
    /// If that is repeated for multiple interfaces,
    /// gathering them all at once amorticizes that cost.
    #[doc(hidden)] // use the Salsa query instead
    pub fn implementers_map(&self) -> HashMap<Name, HashSet<Name>> {
        let mut map = HashMap::<Name, HashSet<Name>>::new();
        for (ty_name, ty) in &self.types {
            let interfaces = match ty {
                ExtendedType::Object(def) => &def.implements_interfaces,
                ExtendedType::Interface(def) => &def.implements_interfaces,
                ExtendedType::Scalar(_)
                | ExtendedType::Union(_)
                | ExtendedType::Enum(_)
                | ExtendedType::InputObject(_) => continue,
            };
            for interface in interfaces {
                map.entry(interface.node.clone())
                    .or_default()
                    .insert(ty_name.clone());
            }
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

    /// Return the meta-fields of the given type
    pub(crate) fn meta_fields_definitions(
        &self,
        type_name: &str,
    ) -> &'static [Component<FieldDefinition>] {
        static ROOT_QUERY_FIELDS: LazyLock<[Component<FieldDefinition>; 3]> = LazyLock::new(|| {
            [
                // __typename: String!
                Component::new(FieldDefinition {
                    description: None,
                    name: Name::new("__typename"),
                    arguments: Vec::new(),
                    ty: Type::new_named("String").non_null(),
                    directives: ast::Directives::new(),
                }),
                // __schema: __Schema!
                Component::new(FieldDefinition {
                    description: None,
                    name: Name::new("__schema"),
                    arguments: Vec::new(),
                    ty: Type::new_named("__Schema").non_null(),
                    directives: ast::Directives::new(),
                }),
                // __type(name: String!): __Type
                Component::new(FieldDefinition {
                    description: None,
                    name: Name::new("__type"),
                    arguments: vec![InputValueDefinition {
                        description: None,
                        name: Name::new("name"),
                        ty: ast::Type::new_named("String").non_null().into(),
                        default_value: None,
                        directives: ast::Directives::new(),
                    }
                    .into()],
                    ty: Type::new_named("__Type"),
                    directives: ast::Directives::new(),
                }),
            ]
        });
        if self
            .schema_definition
            .query
            .as_ref()
            .is_some_and(|n| n == type_name)
        {
            // __typename: String!
            // __schema: __Schema!
            // __type(name: String!): __Type
            ROOT_QUERY_FIELDS.get()
        } else {
            // __typename: String!
            std::slice::from_ref(&ROOT_QUERY_FIELDS.get()[0])
        }
    }

    /// Returns whether the type `ty` is defined as is an input type
    ///
    /// <https://spec.graphql.org/October2021/#sec-Input-and-Output-Types>
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
    /// <https://spec.graphql.org/October2021/#sec-Input-and-Output-Types>
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
    /// Collect `schema` extensions that contribute any component
    ///
    /// The order of the returned set is unspecified but deterministic
    /// for a given apollo-compiler version.
    pub fn extensions(&self) -> IndexSet<&ExtensionId> {
        self.directives
            .iter()
            .flat_map(|dir| dir.origin.extension_id())
            .chain(
                self.query
                    .as_ref()
                    .and_then(|name| name.origin.extension_id()),
            )
            .chain(
                self.mutation
                    .as_ref()
                    .and_then(|name| name.origin.extension_id()),
            )
            .chain(
                self.subscription
                    .as_ref()
                    .and_then(|name| name.origin.extension_id()),
            )
            .collect()
    }
}

impl ExtendedType {
    /// Return the source location of the type's base definition.
    ///
    /// If the type has extensions, those are not covered by this location.
    pub fn location(&self) -> Option<NodeLocation> {
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

    /// Returns true if a value of this type can be used as an input value.
    ///
    /// # Spec
    /// This implements spec function
    /// [`IsInputType(type)`](https://spec.graphql.org/draft/#IsInputType())
    pub fn is_input_type(&self) -> bool {
        matches!(self, Self::Scalar(_) | Self::Enum(_) | Self::InputObject(_))
    }

    /// Returns true if a value of this type can be used as an output value.
    ///
    /// # Spec
    /// This implements spec function
    /// [`IsOutputType(type)`](https://spec.graphql.org/draft/#IsOutputType())
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

    pub fn directives(&self) -> &Directives {
        match self {
            Self::Scalar(ty) => &ty.directives,
            Self::Object(ty) => &ty.directives,
            Self::Interface(ty) => &ty.directives,
            Self::Union(ty) => &ty.directives,
            Self::Enum(ty) => &ty.directives,
            Self::InputObject(ty) => &ty.directives,
        }
    }

    pub fn description(&self) -> Option<&NodeStr> {
        match self {
            Self::Scalar(ty) => ty.description.as_ref(),
            Self::Object(ty) => ty.description.as_ref(),
            Self::Interface(ty) => ty.description.as_ref(),
            Self::Union(ty) => ty.description.as_ref(),
            Self::Enum(ty) => ty.description.as_ref(),
            Self::InputObject(ty) => ty.description.as_ref(),
        }
    }

    serialize_method!();
}

impl ScalarType {
    /// Collect scalar type extensions that contribute any component
    ///
    /// The order of the returned set is unspecified but deterministic
    /// for a given apollo-compiler version.
    pub fn extensions(&self) -> IndexSet<&ExtensionId> {
        self.directives
            .iter()
            .flat_map(|dir| dir.origin.extension_id())
            .collect()
    }

    serialize_method!();
}

impl ObjectType {
    /// Collect object type extensions that contribute any component
    ///
    /// The order of the returned set is unspecified but deterministic
    /// for a given apollo-compiler version.
    pub fn extensions(&self) -> IndexSet<&ExtensionId> {
        self.directives
            .iter()
            .flat_map(|dir| dir.origin.extension_id())
            .chain(
                self.implements_interfaces
                    .iter()
                    .flat_map(|component| component.origin.extension_id()),
            )
            .chain(
                self.fields
                    .values()
                    .flat_map(|field| field.origin.extension_id()),
            )
            .collect()
    }

    serialize_method!();
}

impl InterfaceType {
    /// Collect interface type extensions that contribute any component
    ///
    /// The order of the returned set is unspecified but deterministic
    /// for a given apollo-compiler version.
    pub fn extensions(&self) -> IndexSet<&ExtensionId> {
        self.directives
            .iter()
            .flat_map(|dir| dir.origin.extension_id())
            .chain(
                self.implements_interfaces
                    .iter()
                    .flat_map(|component| component.origin.extension_id()),
            )
            .chain(
                self.fields
                    .values()
                    .flat_map(|field| field.origin.extension_id()),
            )
            .collect()
    }

    serialize_method!();
}

impl UnionType {
    /// Collect union type extensions that contribute any component
    ///
    /// The order of the returned set is unspecified but deterministic
    /// for a given apollo-compiler version.
    pub fn extensions(&self) -> IndexSet<&ExtensionId> {
        self.directives
            .iter()
            .flat_map(|dir| dir.origin.extension_id())
            .chain(
                self.members
                    .iter()
                    .flat_map(|component| component.origin.extension_id()),
            )
            .collect()
    }

    serialize_method!();
}

impl EnumType {
    /// Collect enum type extensions that contribute any component
    ///
    /// The order of the returned set is unspecified but deterministic
    /// for a given apollo-compiler version.
    pub fn extensions(&self) -> IndexSet<&ExtensionId> {
        self.directives
            .iter()
            .flat_map(|dir| dir.origin.extension_id())
            .chain(
                self.values
                    .values()
                    .flat_map(|value| value.origin.extension_id()),
            )
            .collect()
    }

    serialize_method!();
}

impl InputObjectType {
    /// Collect input object type extensions that contribute any component
    ///
    /// The order of the returned set is unspecified but deterministic
    /// for a given apollo-compiler version.
    pub fn extensions(&self) -> IndexSet<&ExtensionId> {
        self.directives
            .iter()
            .flat_map(|dir| dir.origin.extension_id())
            .chain(
                self.fields
                    .values()
                    .flat_map(|field| field.origin.extension_id()),
            )
            .collect()
    }

    serialize_method!();
}

impl Directives {
    pub const fn new() -> Self {
        Self(Vec::new())
    }

    /// Returns an iterator of directives with the given name.
    ///
    /// This method is best for repeatable directives.
    /// See also [`get`][Self::get] for non-repeatable directives.
    pub fn get_all<'def: 'name, 'name>(
        &'def self,
        name: &'name str,
    ) -> impl Iterator<Item = &'def Component<Directive>> + 'name {
        self.0.iter().filter(move |dir| dir.name == name)
    }

    /// Returns the first directive with the given name, if any.
    ///
    /// This method is best for non-repeatable directives.
    /// See also [`get_all`][Self::get_all] for repeatable directives.
    pub fn get(&self, name: &str) -> Option<&Component<Directive>> {
        self.get_all(name).next()
    }

    /// Returns whether there is a directive with the given name
    pub fn has(&self, name: &str) -> bool {
        self.get(name).is_some()
    }

    serialize_method!();
}

impl std::fmt::Debug for Directives {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl std::ops::Deref for Directives {
    type Target = Vec<Component<Directive>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for Directives {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<'a> IntoIterator for &'a Directives {
    type Item = &'a Component<Directive>;

    type IntoIter = std::slice::Iter<'a, Component<Directive>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl<'a> IntoIterator for &'a mut Directives {
    type Item = &'a mut Component<Directive>;

    type IntoIter = std::slice::IterMut<'a, Component<Directive>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter_mut()
    }
}

impl<D> FromIterator<D> for Directives
where
    D: Into<Component<Directive>>,
{
    fn from_iter<T: IntoIterator<Item = D>>(iter: T) -> Self {
        Self(iter.into_iter().map(Into::into).collect())
    }
}

impl Eq for Schema {}

impl PartialEq for Schema {
    fn eq(&self, other: &Self) -> bool {
        let Self {
            sources: _,      // ignored
            build_errors: _, // ignored
            schema_definition: root_operations,
            directive_definitions,
            types,
        } = self;
        *root_operations == other.schema_definition
            && *directive_definitions == other.directive_definitions
            && *types == other.types
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
            build_errors,
            schema_definition,
            directive_definitions,
            types,
        } = self;
        f.debug_struct("Schema")
            .field("sources", sources)
            .field("build_errors", build_errors)
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

// TODO: use `std::sync::LazyLock` when available https://github.com/rust-lang/rust/issues/109736
struct LazyLock<T> {
    value: OnceLock<T>,
    init: fn() -> T,
}

impl<T> LazyLock<T> {
    const fn new(init: fn() -> T) -> Self {
        Self {
            value: OnceLock::new(),
            init,
        }
    }

    fn get(&self) -> &T {
        self.value.get_or_init(self.init)
    }
}
