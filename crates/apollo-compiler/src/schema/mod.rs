//! High-level representation of a GraphQL schema

use crate::ast;
use crate::Arc;
use crate::FileId;
use crate::Node;
use crate::NodeLocation;
use crate::NodeStr;
use crate::Parser;
use crate::SourceFile;
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
#[derive(Debug, Clone)]
pub struct Schema {
    /// Source files, if any, that were parsed to contribute to this schema.
    ///
    /// The schema (including parsed definitions) may have been modified since parsing.
    pub sources: IndexMap<FileId, Arc<SourceFile>>,

    /// Errors that occurred when building this schema,
    /// either parsing a source file or converting from AST.
    pub build_errors: Vec<BuildError>,

    /// The description of the `schema` definition
    pub description: Option<NodeStr>,

    /// Directives applied to the `schema` definition or a `schema` extension
    pub directives: Directives,

    /// Built-in and explicit directive definitions
    pub directive_definitions: IndexMap<Name, Node<DirectiveDefinition>>,

    /// Definitions and extensions of built-in scalars, introspection types,
    /// and explicit types
    pub types: IndexMap<NamedType, ExtendedType>,

    /// Name of the object type for the `query` root operation
    pub query_type: Option<ComponentStr>,

    /// Name of the object type for the `mutation` root operation
    pub mutation_type: Option<ComponentStr>,

    /// Name of the object type for the `subscription` root operation
    pub subscription_type: Option<ComponentStr>,
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
    pub name: Name,
    pub description: Option<NodeStr>,
    pub directives: Directives,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObjectType {
    pub name: Name,
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
    pub name: Name,
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
    pub name: Name,
    pub description: Option<NodeStr>,
    pub directives: Directives,

    /// * Key: name of a member object type
    /// * Value: which union type extension defined this implementation,
    ///   or `None` for the union type definition.
    pub members: IndexSet<ComponentStr>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnumType {
    pub name: Name,
    pub description: Option<NodeStr>,
    pub directives: Directives,
    pub values: IndexMap<Name, Component<EnumValueDefinition>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InputObjectType {
    pub name: Name,
    pub description: Option<NodeStr>,
    pub directives: Directives,
    pub fields: IndexMap<Name, Component<InputValueDefinition>>,
}

/// AST node that has been skipped during conversion to `Schema`
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BuildError {
    /// Found an executable definition, which is unexpected when building a schema.
    ///
    /// If this is intended, use `parse_mixed`.
    UnexpectedExecutableDefinition(ast::Definition),

    /// Found multiple `schema` definitions,
    /// or multiple type or directive definitions with the same name.
    ///
    /// `Definition::*Definition` variant
    DefinitionCollision(ast::Definition),

    /// Found an extension without a corresponding definition to extend
    ///
    /// `Definition::*Extension` variant
    OrphanExtension(ast::Definition),

    DuplicateRootOperation {
        operation_type: ast::OperationType,
        object_type: NamedType,
    },

    DuplicateImplementsInterface {
        implementer_name: NamedType,
        interface_name: Name,
    },

    FieldNameCollision {
        /// Object type or interface type
        type_name: NamedType,
        field: Node<ast::FieldDefinition>,
    },

    EnumValueNameCollision {
        enum_name: NamedType,
        value: Node<ast::EnumValueDefinition>,
    },

    UnionMemberNameCollision {
        union_name: NamedType,
        member: NamedType,
    },

    InputFieldNameCollision {
        type_name: NamedType,
        field: Node<ast::InputValueDefinition>,
    },
}

/// Could not find the requested field definition
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FieldLookupError {
    NoSuchType,
    NoSuchField,
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

    pub fn validate(&self) -> Result<(), Diagnostics> {
        let mut errors = Diagnostics::new(self.sources.clone());
        validation::validate_schema(&mut errors, self);
        errors.into_result()
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
    pub fn root_operation(&self, operation_type: ast::OperationType) -> Option<&ComponentStr> {
        match operation_type {
            ast::OperationType::Query => &self.query_type,
            ast::OperationType::Mutation => &self.mutation_type,
            ast::OperationType::Subscription => &self.subscription_type,
        }
        .as_ref()
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
                self.query_type
                    .as_ref()
                    .and_then(|name| name.origin.extension_id()),
            )
            .chain(
                self.mutation_type
                    .as_ref()
                    .and_then(|name| name.origin.extension_id()),
            )
            .chain(
                self.subscription_type
                    .as_ref()
                    .and_then(|name| name.origin.extension_id()),
            )
            .collect()
    }

    /// Returns the definition of a type’s explicit field or meta-field.
    pub fn type_field(
        &self,
        type_name: &str,
        field_name: &str,
    ) -> Result<&Component<FieldDefinition>, FieldLookupError> {
        let ty_def = self
            .types
            .get(type_name)
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
            .ok_or(FieldLookupError::NoSuchField)
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
            .query_type
            .as_ref()
            .is_some_and(|n| n.node == type_name)
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

impl ExtendedType {
    /// Return the name of the type.
    pub fn name(&self) -> &ast::Name {
        match self {
            Self::Scalar(ty) => &ty.name,
            Self::Object(ty) => &ty.name,
            Self::Interface(ty) => &ty.name,
            Self::Union(ty) => &ty.name,
            Self::Enum(ty) => &ty.name,
            Self::InputObject(ty) => &ty.name,
        }
    }

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
    /// This implements spec function `IsInputType(type)`: https://spec.graphql.org/draft/#IsInputType()
    pub fn is_input_type(&self) -> bool {
        matches!(self, Self::Scalar(_) | Self::Enum(_) | Self::InputObject(_))
    }

    /// Returns true if a value of this type can be used as an output value.
    ///
    /// # Spec
    /// This implements spec function `IsOutputType(type)`: https://spec.graphql.org/draft/#IsOutputType()
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
    pub fn new() -> Self {
        Self(Vec::new())
    }

    /// Returns an iterator of directives with the given name.
    ///
    /// This method is best for repeatable directives. For non-repeatable directives,
    /// see [`directive_by_name`][Self::directive_by_name] (singular)
    pub fn get_all<'def: 'name, 'name>(
        &'def self,
        name: &'name str,
    ) -> impl Iterator<Item = &'def Component<Directive>> + 'name {
        self.0.iter().filter(move |dir| dir.name == name)
    }

    /// Returns the first directive with the given name, if any.
    ///
    /// This method is best for non-repeatable directives. For repeatable directives,
    /// see [`directives_by_name`][Self::directives_by_name] (plural)
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
            sources: _,
            build_errors: _,
            description,
            directives,
            directive_definitions,
            types,
            query_type,
            mutation_type,
            subscription_type,
        } = self;
        *description == other.description
            && *directives == other.directives
            && *directive_definitions == other.directive_definitions
            && *types == other.types
            && *query_type == other.query_type
            && *mutation_type == other.mutation_type
            && *subscription_type == other.subscription_type
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
