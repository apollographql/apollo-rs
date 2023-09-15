//! High-level representation of a GraphQL schema

use crate::ast;
use crate::FileId;
use crate::Node;
use crate::NodeLocation;
use crate::NodeStr;
use indexmap::IndexMap;
use indexmap::IndexSet;
use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::OnceLock;

mod component;
mod from_ast;
mod serialize;
#[cfg(test)]
mod tests;

pub use self::component::{Component, ComponentOrigin, ComponentStr, ExtensionId};
pub use self::from_ast::SchemaBuilder;
pub use crate::ast::{
    Directive, DirectiveDefinition, DirectiveLocation, EnumValueDefinition, FieldDefinition,
    InputValueDefinition, Name, NamedType, Type, Value,
};

/// High-level representation of a GraphQL schema
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Schema {
    /// The description of the `schema` definition
    pub description: Option<NodeStr>,

    /// Directives applied to the `schema` definition or a `schema` extension
    pub directives: Vec<Component<Directive>>,

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScalarType {
    pub name: Name,
    pub description: Option<NodeStr>,
    pub directives: Vec<Component<Directive>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObjectType {
    pub name: Name,
    pub description: Option<NodeStr>,
    pub implements_interfaces: IndexMap<Name, ComponentOrigin>,
    pub directives: Vec<Component<Directive>>,

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
    pub implements_interfaces: IndexMap<Name, ComponentOrigin>,

    pub directives: Vec<Component<Directive>>,

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
    pub directives: Vec<Component<Directive>>,

    /// * Key: name of a member object type
    /// * Value: which union type extension defined this implementation,
    ///   or `None` for the union type definition.
    pub members: IndexMap<NamedType, ComponentOrigin>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnumType {
    pub name: Name,
    pub description: Option<NodeStr>,
    pub directives: Vec<Component<Directive>>,
    pub values: IndexMap<Name, Component<EnumValueDefinition>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InputObjectType {
    pub name: Name,
    pub description: Option<NodeStr>,
    pub directives: Vec<Component<Directive>>,
    pub fields: IndexMap<Name, Component<InputValueDefinition>>,
}

/// Could not find the requested field definition
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FieldLookupError {
    NoSuchType,
    NoSuchField,
}

macro_rules! directive_by_name_method {
    () => {
        /// Returns the first directive with the given name, if any.
        ///
        /// This method is best for non-repeatable directives. For repeatable directives,
        /// see [`directives_by_name`][Self::directives_by_name] (plural)
        pub fn directive_by_name(&self, name: &str) -> Option<&Component<Directive>> {
            self.directives_by_name(name).next()
        }
    };
}

fn directives_by_name<'def: 'name, 'name>(
    directives: &'def [Component<Directive>],
    name: &'name str,
) -> impl Iterator<Item = &'def Component<Directive>> + 'name {
    directives.iter().filter(move |dir| dir.name == name)
}

macro_rules! directive_methods {
    () => {
        /// Returns an iterator of directives with the given name.
        ///
        /// This method is best for repeatable directives. For non-repeatable directives,
        /// see [`directive_by_name`][Self::directive_by_name] (singular)
        pub fn directives_by_name<'def: 'name, 'name>(
            &'def self,
            name: &'name str,
        ) -> impl Iterator<Item = &'def Component<Directive>> + 'name {
            directives_by_name(&self.directives, name)
        }

        directive_by_name_method!();
    };
}

impl Schema {
    /// Returns a new builder for creating a Schema from AST documents,
    /// initialized with built-in directives, built-in scalars, and introspection types
    pub fn builder() -> SchemaBuilder {
        SchemaBuilder::new()
    }

    /// Returns an (almost) empty schema.
    ///
    /// It starts with built-in directives, built-in scalars, and introspection types.
    /// It can then be filled programatically.
    #[allow(clippy::new_without_default)] // not a great implicit default in generic contexts
    pub fn new() -> Self {
        let (schema, _orphan_definitions) = SchemaBuilder::new().build();
        // _orphan_definitions already debug_assert’ed empty in SchemaBuilder::new
        schema
    }

    /// Returns a schema built from one AST document
    ///
    /// The schema also contains built-in directives, built-in scalars, and introspection types.
    ///
    /// Additionally, orphan extensions that are not represented in `Schema`
    /// are returned separately:
    ///
    /// * `Definition::SchemaExtension` variants if no `Definition::SchemaDefinition` was found
    /// * `Definition::*TypeExtension` if no `Definition::*TypeDefinition` with the same name
    ///   was found, or if it is a different kind of type
    pub fn from_ast(document: &ast::Document) -> (Self, impl Iterator<Item = ast::Definition>) {
        let mut builder = SchemaBuilder::new();
        builder.add_document(document);
        builder.build()
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
            for interface in interfaces.keys() {
                map.entry(interface.clone())
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
                .contains_key(abstract_type)
            }),
            ExtendedType::Union(def) => def.members.contains_key(maybe_subtype),
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
                    directives: Vec::new(),
                }),
                // __schema: __Schema!
                Component::new(FieldDefinition {
                    description: None,
                    name: Name::new("__schema"),
                    arguments: Vec::new(),
                    ty: Type::new_named("__Schema").non_null(),
                    directives: Vec::new(),
                }),
                // __type(name: String!): __Type
                Component::new(FieldDefinition {
                    description: None,
                    name: Name::new("__type"),
                    arguments: vec![InputValueDefinition {
                        description: None,
                        name: Name::new("name"),
                        ty: ast::Type::new_named("String").non_null(),
                        default_value: None,
                        directives: Vec::new(),
                    }
                    .into()],
                    ty: Type::new_named("__Type"),
                    directives: Vec::new(),
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

    directive_methods!();
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
    pub fn location(&self) -> Option<&'_ NodeLocation> {
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

    /// Returns an iterator of directives with the given name.
    ///
    /// This method is best for repeatable directives. For non-repeatable directives,
    /// see [`directive_by_name`][Self::directive_by_name] (singular)
    pub fn directives_by_name<'def: 'name, 'name>(
        &'def self,
        name: &'name str,
    ) -> impl Iterator<Item = &'def Component<Directive>> + 'name {
        match self {
            Self::Scalar(ty) => directives_by_name(&ty.directives, name),
            Self::Object(ty) => directives_by_name(&ty.directives, name),
            Self::Interface(ty) => directives_by_name(&ty.directives, name),
            Self::Union(ty) => directives_by_name(&ty.directives, name),
            Self::Enum(ty) => directives_by_name(&ty.directives, name),
            Self::InputObject(ty) => directives_by_name(&ty.directives, name),
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

    directive_by_name_method!();
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

    directive_methods!();
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
                    .values()
                    .flat_map(|origin| origin.extension_id()),
            )
            .chain(
                self.fields
                    .values()
                    .flat_map(|field| field.origin.extension_id()),
            )
            .collect()
    }

    directive_methods!();
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
                    .values()
                    .flat_map(|origin| origin.extension_id()),
            )
            .chain(
                self.fields
                    .values()
                    .flat_map(|field| field.origin.extension_id()),
            )
            .collect()
    }

    directive_methods!();
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
                    .values()
                    .flat_map(|origin| origin.extension_id()),
            )
            .collect()
    }

    directive_methods!();
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

    directive_methods!();
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

    directive_methods!();
    serialize_method!();
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
