//! *Abstract Syntax Tree* for GraphQL documents.
//! Lower-level than [`Schema`][crate::Schema]
//! or [`ExecutableDocument`][crate::ExecutableDocument].
//!
//! This AST aims to faithfully represent documents that conform to the GraphQL
//! [syntactic grammar], except for their [ignored tokens].
//! These documents may or may not be [valid].
//!
//! Parsing an input that does not conform to the grammar results in parse errors
//! together with a partial AST.
//!
//! [ignored tokens]: https://spec.graphql.org/October2021/#Ignored
//! [syntactic grammar]: https://spec.graphql.org/October2021/#sec-Language
//! [valid]: https://spec.graphql.org/October2021/#sec-Validation
//!
//! ## Parsing
//!
//! Start with [`Document::parse`], or [`Parser`][crate::parser::Parser]
//! to change the parser configuration.
//!
//! ## Structural sharing and mutation
//!
//! Nodes inside documents are wrapped in [`Node`], a reference-counted smart pointer.
//! This allows sharing nodes between documents without cloning entire subtrees.
//! To modify a node, the [`make_mut`][Node::make_mut] method provides copy-on-write semantics.
//!
//! ## Serialization
//!
//! [`Document`] and its node types implement [`Display`][std::fmt::Display]
//! and [`ToString`] by serializing to GraphQL syntax with a default configuration.
//! [`serialize`][Document::serialize] methods return a builder
//! that has chaining methods for setting serialization configuration,
//! and also implements `Display` and `ToString`.

use crate::parser::SourceMap;
use crate::Name;
use crate::Node;

pub(crate) mod from_cst;
pub(crate) mod impls;
pub(crate) mod serialize;

pub use self::serialize::Serialize;

#[derive(Clone)]
pub struct Document {
    /// If this document was originally parsed from a source file,
    /// this map contains one entry for that file and its ID.
    ///
    /// The document may have been modified since.
    pub sources: SourceMap,

    pub definitions: Vec<Definition>,
}

const _: () = {
    const fn assert_send<T: Send>() {}
    const fn assert_sync<T: Sync>() {}
    assert_send::<Document>();
    assert_sync::<Document>();
};

/// Refers to the name of a GraphQL type defined elsewhere
pub type NamedType = Name;

#[derive(Clone, Eq, PartialEq, Hash)]
pub enum Definition {
    OperationDefinition(Node<OperationDefinition>),
    FragmentDefinition(Node<FragmentDefinition>),
    DirectiveDefinition(Node<DirectiveDefinition>),
    SchemaDefinition(Node<SchemaDefinition>),
    ScalarTypeDefinition(Node<ScalarTypeDefinition>),
    ObjectTypeDefinition(Node<ObjectTypeDefinition>),
    InterfaceTypeDefinition(Node<InterfaceTypeDefinition>),
    UnionTypeDefinition(Node<UnionTypeDefinition>),
    EnumTypeDefinition(Node<EnumTypeDefinition>),
    InputObjectTypeDefinition(Node<InputObjectTypeDefinition>),
    SchemaExtension(Node<SchemaExtension>),
    ScalarTypeExtension(Node<ScalarTypeExtension>),
    ObjectTypeExtension(Node<ObjectTypeExtension>),
    InterfaceTypeExtension(Node<InterfaceTypeExtension>),
    UnionTypeExtension(Node<UnionTypeExtension>),
    EnumTypeExtension(Node<EnumTypeExtension>),
    InputObjectTypeExtension(Node<InputObjectTypeExtension>),
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct OperationDefinition {
    pub operation_type: OperationType,
    pub name: Option<Name>,
    pub variables: Vec<Node<VariableDefinition>>,
    pub directives: DirectiveList,
    pub selection_set: Vec<Selection>,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct FragmentDefinition {
    pub name: Name,
    pub type_condition: NamedType,
    pub directives: DirectiveList,
    pub selection_set: Vec<Selection>,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct DirectiveDefinition {
    pub description: Option<Node<str>>,
    pub name: Name,
    pub arguments: Vec<Node<InputValueDefinition>>,
    pub repeatable: bool,
    pub locations: Vec<DirectiveLocation>,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct SchemaDefinition {
    pub description: Option<Node<str>>,
    pub directives: DirectiveList,
    pub root_operations: Vec<Node<(OperationType, NamedType)>>,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct ScalarTypeDefinition {
    pub description: Option<Node<str>>,
    pub name: Name,
    pub directives: DirectiveList,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct ObjectTypeDefinition {
    pub description: Option<Node<str>>,
    pub name: Name,
    pub implements_interfaces: Vec<Name>,
    pub directives: DirectiveList,
    pub fields: Vec<Node<FieldDefinition>>,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct InterfaceTypeDefinition {
    pub description: Option<Node<str>>,
    pub name: Name,
    pub implements_interfaces: Vec<Name>,
    pub directives: DirectiveList,
    pub fields: Vec<Node<FieldDefinition>>,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct UnionTypeDefinition {
    pub description: Option<Node<str>>,
    pub name: Name,
    pub directives: DirectiveList,
    pub members: Vec<NamedType>,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct EnumTypeDefinition {
    pub description: Option<Node<str>>,
    pub name: Name,
    pub directives: DirectiveList,
    pub values: Vec<Node<EnumValueDefinition>>,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct InputObjectTypeDefinition {
    pub description: Option<Node<str>>,
    pub name: Name,
    pub directives: DirectiveList,
    pub fields: Vec<Node<InputValueDefinition>>,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct SchemaExtension {
    pub directives: DirectiveList,
    pub root_operations: Vec<Node<(OperationType, NamedType)>>,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct ScalarTypeExtension {
    pub name: Name,
    pub directives: DirectiveList,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct ObjectTypeExtension {
    pub name: Name,
    pub implements_interfaces: Vec<Name>,
    pub directives: DirectiveList,
    pub fields: Vec<Node<FieldDefinition>>,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct InterfaceTypeExtension {
    pub name: Name,
    pub implements_interfaces: Vec<Name>,
    pub directives: DirectiveList,
    pub fields: Vec<Node<FieldDefinition>>,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct UnionTypeExtension {
    pub name: Name,
    pub directives: DirectiveList,
    pub members: Vec<NamedType>,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct EnumTypeExtension {
    pub name: Name,
    pub directives: DirectiveList,
    pub values: Vec<Node<EnumValueDefinition>>,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct InputObjectTypeExtension {
    pub name: Name,
    pub directives: DirectiveList,
    pub fields: Vec<Node<InputValueDefinition>>,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Argument {
    pub name: Name,
    pub value: Node<Value>,
}

#[derive(Clone, Eq, PartialEq, Hash, Default)]
pub struct DirectiveList(pub Vec<Node<Directive>>);

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Directive {
    pub name: Name,
    pub arguments: Vec<Node<Argument>>,
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum OperationType {
    Query,
    Mutation,
    Subscription,
}

#[derive(Copy, Clone, Hash, PartialEq, Eq)]
pub enum DirectiveLocation {
    Query,
    Mutation,
    Subscription,
    Field,
    FragmentDefinition,
    FragmentSpread,
    InlineFragment,
    VariableDefinition,
    Schema,
    Scalar,
    Object,
    FieldDefinition,
    ArgumentDefinition,
    Interface,
    Union,
    Enum,
    EnumValue,
    InputObject,
    InputFieldDefinition,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct VariableDefinition {
    pub name: Name,
    pub ty: Node<Type>,
    pub default_value: Option<Node<Value>>,
    pub directives: DirectiveList,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, serde::Serialize, serde::Deserialize)]
pub enum Type {
    Named(NamedType),
    NonNullNamed(NamedType),
    List(Box<Type>),
    NonNullList(Box<Type>),
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct FieldDefinition {
    pub description: Option<Node<str>>,
    pub name: Name,
    pub arguments: Vec<Node<InputValueDefinition>>,
    pub ty: Type,
    pub directives: DirectiveList,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct InputValueDefinition {
    pub description: Option<Node<str>>,
    pub name: Name,
    pub ty: Node<Type>,
    pub default_value: Option<Node<Value>>,
    pub directives: DirectiveList,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct EnumValueDefinition {
    pub description: Option<Node<str>>,
    pub value: Name,
    pub directives: DirectiveList,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum Selection {
    Field(Node<Field>),
    FragmentSpread(Node<FragmentSpread>),
    InlineFragment(Node<InlineFragment>),
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Field {
    pub alias: Option<Name>,
    pub name: Name,
    pub arguments: Vec<Node<Argument>>,
    pub directives: DirectiveList,
    pub selection_set: Vec<Selection>,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct FragmentSpread {
    pub fragment_name: Name,
    pub directives: DirectiveList,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct InlineFragment {
    pub type_condition: Option<NamedType>,
    pub directives: DirectiveList,
    pub selection_set: Vec<Selection>,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, serde::Serialize, serde::Deserialize)]
pub enum Value {
    Null,
    Enum(Name),
    Variable(Name),
    String(
        /// The value after escape sequences are resolved
        String,
    ),
    Float(FloatValue),
    Int(IntValue),
    Boolean(bool),
    List(Vec<Node<Value>>),
    Object(Vec<(Name, Node<Value>)>),
}

#[derive(Clone, Eq, PartialEq, Hash)]
pub struct IntValue(String);

#[derive(Clone, Eq, PartialEq, Hash)]
pub struct FloatValue(String);

/// `IntValue` or `FloatValue` magnitude too large to be converted to `f64`.
#[derive(Clone, Eq, PartialEq)]
#[non_exhaustive]
pub struct FloatOverflowError {}
