//! High-level representation of a GraphQL schema

use crate::ast;
use crate::collections::HashMap;
use crate::collections::HashSet;
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
pub use self::component::ComponentOrigin;
pub use self::component::ExtensionId;
pub use self::from_ast::SchemaBuilder;
pub use crate::ast::Directive;
pub use crate::ast::DirectiveDefinition;
pub use crate::ast::DirectiveLocation;
pub use crate::ast::EnumValueDefinition;
pub use crate::ast::FieldDefinition;
pub use crate::ast::InputValueDefinition;
pub use crate::ast::NamedType;
pub use crate::ast::Type;
pub use crate::ast::Value;

/// High-level representation of a GraphQL schema
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

    /// Definitions and extensions of built-in scalars, introspection types,
    /// and explicit types
    pub types: IndexMap<NamedType, ExtendedType>,
}

/// The `schema` definition and its extensions, defining root operations
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SchemaDefinition {
    pub description: Option<Node<str>>,
    pub directives: DirectiveList,

    /// Name of the object type for the `query` root operation
    pub query: Option<ComponentName>,

    /// Name of the object type for the `mutation` root operation
    pub mutation: Option<ComponentName>,

    /// Name of the object type for the `subscription` root operation
    pub subscription: Option<ComponentName>,
}

#[derive(Clone, Eq, PartialEq, Hash, Default)]
pub struct DirectiveList(pub Vec<Component<Directive>>);

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
    pub description: Option<Node<str>>,
    pub name: Name,
    pub directives: DirectiveList,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObjectType {
    pub description: Option<Node<str>>,
    pub name: Name,
    pub implements_interfaces: IndexSet<ComponentName>,
    pub directives: DirectiveList,

    /// Explicit field definitions.
    ///
    /// When looking up a definition,
    /// consider using [`Schema::type_field`] instead to include meta-fields.
    pub fields: IndexMap<Name, Component<FieldDefinition>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InterfaceType {
    pub description: Option<Node<str>>,
    pub name: Name,
    pub implements_interfaces: IndexSet<ComponentName>,

    pub directives: DirectiveList,

    /// Explicit field definitions.
    ///
    /// When looking up a definition,
    /// consider using [`Schema::type_field`] instead to include meta-fields.
    pub fields: IndexMap<Name, Component<FieldDefinition>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnionType {
    pub description: Option<Node<str>>,
    pub name: Name,
    pub directives: DirectiveList,

    /// * Key: name of a member object type
    /// * Value: which union type extension defined this implementation,
    ///   or `None` for the union type definition.
    pub members: IndexSet<ComponentName>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnumType {
    pub description: Option<Node<str>>,
    pub name: Name,
    pub directives: DirectiveList,
    pub values: IndexMap<Name, Component<EnumValueDefinition>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InputObjectType {
    pub description: Option<Node<str>>,
    pub name: Name,
    pub directives: DirectiveList,
    pub fields: IndexMap<Name, Component<InputValueDefinition>>,
}

/// A collection of type names that implement an interface.
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
    /// Names of the concrete types that implement an interface.
    pub objects: HashSet<Name>,
    /// Names of the interfaces that implement an interface.
    pub interfaces: HashSet<Name>,
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
        SchemaBuilder::new().build().unwrap()
    }

    /// Parse a single source file into a schema, with the default parser configuration.
    ///
    /// Create a [`Parser`] to use different parser configuration.
    /// Use [`builder()`][Self::builder] to build a schema from multiple parsed files.
    pub fn parse(
        source_text: impl Into<String>,
        path: impl AsRef<Path>,
    ) -> Result<Self, WithErrors<Self>> {
        Parser::default().parse_schema(source_text, path)
    }

    /// [`parse`][Self::parse] then [`validate`][Self::validate],
    /// to get a `Valid<Schema>` when mutating it isn’t needed.
    pub fn parse_and_validate(
        source_text: impl Into<String>,
        path: impl AsRef<Path>,
    ) -> Result<Valid<Self>, WithErrors<Self>> {
        let mut builder = Schema::builder();
        Parser::default().parse_into_schema_builder(source_text, path, &mut builder);
        let (schema, mut errors) = builder.build_inner();
        validation::validate_schema(&mut errors, &schema);
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

    pub fn validate(self) -> Result<Valid<Self>, WithErrors<Self>> {
        let mut errors = DiagnosticList::new(self.sources.clone());
        validation::validate_schema(&mut errors, &self);
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
        .map(|component| &component.name)
    }

    /// Returns the definition of a type’s explicit field or meta-field.
    pub fn type_field(
        &self,
        type_name: &str,
        field_name: &str,
    ) -> Result<&Component<FieldDefinition>, FieldLookupError<'_>> {
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
                        map.entry(interface.name.clone())
                            .or_default()
                            .objects
                            .insert(ty_name.clone());
                    }
                }
                ExtendedType::Interface(def) => {
                    for interface in &def.implements_interfaces {
                        map.entry(interface.name.clone())
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
    pub fn iter_root_operations(
        &self,
    ) -> impl Iterator<Item = (ast::OperationType, &ComponentName)> {
        [
            (ast::OperationType::Query, &self.query),
            (ast::OperationType::Mutation, &self.mutation),
            (ast::OperationType::Subscription, &self.subscription),
        ]
        .into_iter()
        .filter_map(|(ty, maybe_op)| maybe_op.as_ref().map(|op| (ty, op)))
    }

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

impl DirectiveList {
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

    pub(crate) fn iter_ast(&self) -> impl Iterator<Item = &Node<ast::Directive>> {
        self.0.iter().map(|component| &component.node)
    }

    serialize_method!();
}

impl std::fmt::Debug for DirectiveList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl std::ops::Deref for DirectiveList {
    type Target = Vec<Component<Directive>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for DirectiveList {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl IntoIterator for DirectiveList {
    type Item = Component<Directive>;

    type IntoIter = std::vec::IntoIter<Component<Directive>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a> IntoIterator for &'a DirectiveList {
    type Item = &'a Component<Directive>;

    type IntoIter = std::slice::Iter<'a, Component<Directive>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl<'a> IntoIterator for &'a mut DirectiveList {
    type Item = &'a mut Component<Directive>;

    type IntoIter = std::slice::IterMut<'a, Component<Directive>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter_mut()
    }
}

impl<D> FromIterator<D> for DirectiveList
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
            sources: _, // ignored
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
    __typename: Component<FieldDefinition>,
    __schema: Component<FieldDefinition>,
    __type: Component<FieldDefinition>,
}

impl MetaFieldDefinitions {
    fn get() -> &'static Self {
        static DEFS: OnceLock<MetaFieldDefinitions> = OnceLock::new();
        DEFS.get_or_init(|| Self {
            // __typename: String!
            __typename: Component::new(FieldDefinition {
                description: None,
                name: name!("__typename"),
                arguments: Vec::new(),
                ty: ty!(String!),
                directives: ast::DirectiveList::new(),
            }),
            // __schema: __Schema!
            __schema: Component::new(FieldDefinition {
                description: None,
                name: name!("__schema"),
                arguments: Vec::new(),
                ty: ty!(__Schema!),
                directives: ast::DirectiveList::new(),
            }),
            // __type(name: String!): __Type
            __type: Component::new(FieldDefinition {
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
