//! *Abstract Syntax Tree* for GraphQL documents.
//! An AST [`Document`] is more permissive but lower-level than [`Schema`][crate::Schema]
//! or [`ExecutableDocument`][crate::ExecutableDocument].
//!
//! This AST aims to faithfully represent documents
//! that conform to the GraphQL [syntactic grammar],
//! except that [ignored tokens] such as whitespace are not preserved.
//! These documents may or may not be [valid].
//!
//! Parsing an input that does not conform to the grammar results in parse errors
//! together with a partial AST.
//!
//! [ignored tokens]: https://spec.graphql.org/September2025/#Ignored
//! [syntactic grammar]: https://spec.graphql.org/September2025/#sec-Language
//! [valid]: https://spec.graphql.org/September2025/#sec-Validation
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
//!
//! ## Example
//!
//! ```
//! use apollo_compiler::{ast, name};
//!
//! let source = "{field}";
//! let mut doc = ast::Document::parse(source, "example.graphql").unwrap();
//! for def in &mut doc.definitions {
//!     if let ast::Definition::OperationDefinition(op) = def {
//!         // `op` has type `&mut Node<ast::OperationDefinition>`
//!         // `Node` implements `Deref` but not `DeferMut`
//!         // `make_mut()` clones if necessary and returns `&mut ast::OperationDefinition`
//!         op.make_mut().directives.push(ast::Directive::new(name!(dir)));
//!     }
//! }
//! assert_eq!(doc.serialize().no_indent().to_string(), "query @dir { field }")
//! ```

use crate::collections::IndexSet;
use crate::parser::SourceMap;
use crate::Name;
use crate::Node;
use std::hash::Hash;

pub(crate) mod from_cst;
pub(crate) mod impls;
pub(crate) mod serialize;

pub use self::serialize::Serialize;

/// AST for a GraphQL [_Document_](https://spec.graphql.org/September2025/#Document)
/// that can contain executable definitions, type system (schema) definitions, or both.
///
/// It is typically parsed from one `&str` input “file” but can be also be synthesized
/// programatically.
#[derive(Clone)]
pub struct Document {
    /// If this document was originally parsed from a source file,
    /// this map contains one entry for that file and its ID.
    ///
    /// The document is [mutable][crate::ast#structural-sharing-and-mutation]
    /// so it may have been modified since.
    pub sources: SourceMap,

    pub definitions: Vec<Definition>,
}

const _: () = {
    const fn assert_send<T: Send>() {}
    const fn assert_sync<T: Sync>() {}
    assert_send::<Document>();
    assert_sync::<Document>();
};

/// A [_NamedType_](https://spec.graphql.org/September2025/#NamedType)
/// references by name a GraphQL type defined elsewhere.
pub type NamedType = Name;

/// AST for a top-level [_Definition_](https://spec.graphql.org/September2025/#Definition) of any kind:
/// executable, type system, or type system extension.
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

/// Executable AST for an
/// [_OperationDefinition_](https://spec.graphql.org/September2025/#OperationDefinition).
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct OperationDefinition {
    pub description: Option<Node<str>>,
    pub operation_type: OperationType,
    pub name: Option<Name>,
    pub variables: Vec<Node<VariableDefinition>>,
    pub directives: DirectiveList,
    pub selection_set: Vec<Selection>,
}

/// Executable AST for a
/// [_FragmentDefinition_](https://spec.graphql.org/September2025/#FragmentDefinition).
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct FragmentDefinition {
    pub description: Option<Node<str>>,
    pub name: Name,
    pub type_condition: NamedType,
    pub directives: DirectiveList,
    pub selection_set: Vec<Selection>,
}

/// Type system AST for a `directive @foo`
/// [_DirectiveDefinition_](https://spec.graphql.org/September2025/#DirectiveDefinition).
#[derive(Clone, Debug)]
pub struct DirectiveDefinition {
    pub description: Option<Node<str>>,
    pub name: Name,
    pub arguments: Vec<Node<InputValueDefinition>>,
    pub repeatable: bool,
    pub locations: IndexSet<DirectiveLocation>,
}

impl PartialEq for DirectiveDefinition {
    fn eq(&self, other: &Self) -> bool {
        self.description == other.description
            && self.name == other.name
            && self.repeatable == other.repeatable
            && self.locations == other.locations
            && input_value_definitions_eq(&self.arguments, &other.arguments)
    }
}

impl Eq for DirectiveDefinition {}

impl std::hash::Hash for DirectiveDefinition {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.description.hash(state);
        self.name.hash(state);
        self.repeatable.hash(state);
        // IndexSet: order-independent hash via commutative XOR
        self.locations.len().hash(state);
        let mut locations_hash = 0u64;
        for loc in &self.locations {
            let mut h = std::hash::DefaultHasher::new();
            loc.hash(&mut h);
            locations_hash ^= std::hash::Hasher::finish(&h);
        }
        locations_hash.hash(state);
        input_value_definitions_hash(&self.arguments, state);
    }
}

/// Type system AST for a `schema`
/// [_SchemaDefinition_](https://spec.graphql.org/September2025/#SchemaDefinition).
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct SchemaDefinition {
    pub description: Option<Node<str>>,
    pub directives: DirectiveList,
    pub root_operations: Vec<Node<(OperationType, NamedType)>>,
}

/// Type system AST for a `scalar FooS`
/// [_ScalarTypeDefinition_](https://spec.graphql.org/September2025/#ScalarTypeDefinition).
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct ScalarTypeDefinition {
    pub description: Option<Node<str>>,
    pub name: Name,
    pub directives: DirectiveList,
}

/// Type system AST for a `type FooO`
/// [_ObjectTypeDefinition_](https://spec.graphql.org/September2025/#ObjectTypeDefinition).
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct ObjectTypeDefinition {
    pub description: Option<Node<str>>,
    pub name: Name,
    pub implements_interfaces: Vec<Name>,
    pub directives: DirectiveList,
    pub fields: Vec<Node<FieldDefinition>>,
}

/// Type system AST for an `interface FooI`
/// [_InterfaceTypeDefinition_](https://spec.graphql.org/September2025/#InterfaceTypeDefinition).
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct InterfaceTypeDefinition {
    pub description: Option<Node<str>>,
    pub name: Name,
    pub implements_interfaces: Vec<Name>,
    pub directives: DirectiveList,
    pub fields: Vec<Node<FieldDefinition>>,
}

/// Type system AST for a `union FooU`
/// [_UnionTypeDefinition_](https://spec.graphql.org/September2025/#UnionTypeDefinition).
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct UnionTypeDefinition {
    pub description: Option<Node<str>>,
    pub name: Name,
    pub directives: DirectiveList,
    pub members: Vec<NamedType>,
}

/// Type system AST for an `enum FooE`
/// [_EnumTypeDefinition_](https://spec.graphql.org/September2025/#EnumTypeDefinition).
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct EnumTypeDefinition {
    pub description: Option<Node<str>>,
    pub name: Name,
    pub directives: DirectiveList,
    pub values: Vec<Node<EnumValueDefinition>>,
}

/// Type system AST for an `input FooIn`
/// [_InputObjectTypeDefinition_](https://spec.graphql.org/September2025/#InputObjectTypeDefinition).
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct InputObjectTypeDefinition {
    pub description: Option<Node<str>>,
    pub name: Name,
    pub directives: DirectiveList,
    pub fields: Vec<Node<InputValueDefinition>>,
}

/// Type system AST for an `extend schema`
/// [_SchemaExtension_](https://spec.graphql.org/September2025/#SchemaExtension).
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct SchemaExtension {
    pub directives: DirectiveList,
    pub root_operations: Vec<Node<(OperationType, NamedType)>>,
}

/// Type system AST for an `extend scalar FooS`
/// [_ScalarTypeExtension_](https://spec.graphql.org/September2025/#ScalarTypeExtension).
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct ScalarTypeExtension {
    pub name: Name,
    pub directives: DirectiveList,
}

/// Type system AST for an `extend type FooO`
/// [_ObjectTypeExtension_](https://spec.graphql.org/September2025/#ObjectTypeExtension).
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct ObjectTypeExtension {
    pub name: Name,
    pub implements_interfaces: Vec<Name>,
    pub directives: DirectiveList,
    pub fields: Vec<Node<FieldDefinition>>,
}

/// Type system AST for an `extend interface FooI`
/// [_InterfaceTypeExtension_](https://spec.graphql.org/September2025/#InterfaceTypeExtension).
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct InterfaceTypeExtension {
    pub name: Name,
    pub implements_interfaces: Vec<Name>,
    pub directives: DirectiveList,
    pub fields: Vec<Node<FieldDefinition>>,
}

/// Type system AST for an `extend union FooU`
/// [_UnionTypeExtension_](https://spec.graphql.org/September2025/#UnionTypeExtension).
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct UnionTypeExtension {
    pub name: Name,
    pub directives: DirectiveList,
    pub members: Vec<NamedType>,
}

/// Type system AST for an `extend enum FooE`
/// [_EnumTypeExtension_](https://spec.graphql.org/September2025/#EnumTypeExtension).
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct EnumTypeExtension {
    pub name: Name,
    pub directives: DirectiveList,
    pub values: Vec<Node<EnumValueDefinition>>,
}

/// Type system AST for an `extend input FooIn`
/// [_InputObjectTypeExtension_](https://spec.graphql.org/September2025/#InputObjectTypeExtension).
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct InputObjectTypeExtension {
    pub name: Name,
    pub directives: DirectiveList,
    pub fields: Vec<Node<InputValueDefinition>>,
}

/// AST for an [_Argument_](https://spec.graphql.org/September2025/#Argument)
/// of a [`Field`] selection or [`Directive`] application.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Argument {
    pub name: Name,
    pub value: Node<Value>,
}

/// The list of [_Directives_](https://spec.graphql.org/September2025/#Directives)
/// applied to some context.
///
/// This type is used in both AST and high-level [`Schema`][crate::Schema]
/// representations. In a schema, each directive [`Node`] tracks whether it
/// comes from the “main” definition or from an extension
/// through its [`origin`][Node::origin].
#[derive(Clone, Eq, PartialEq, Hash, Default)]
pub struct DirectiveList(pub Vec<Node<Directive>>);

/// AST for a [_Directive_](https://spec.graphql.org/September2025/#Directive) application.
#[derive(Clone, Debug)]
pub struct Directive {
    pub name: Name,
    pub arguments: Vec<Node<Argument>>,
}

impl PartialEq for Directive {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && arguments_eq(&self.arguments, &other.arguments)
    }
}

impl Eq for Directive {}

impl std::hash::Hash for Directive {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        arguments_hash(&self.arguments, state);
    }
}

/// AST for the [_OperationType_](https://spec.graphql.org/September2025/#OperationType)
/// of an [`OperationDefinition`] or [`RootOperationDefinition`][SchemaDefinition::root_operations].
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum OperationType {
    Query,
    Mutation,
    Subscription,
}

/// AST for a [_DirectiveLocation_](https://spec.graphql.org/September2025/#DirectiveLocation)
/// of a [`DirectiveDefinition`].
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

/// Executable AST for a [_VariableDefinition_](https://spec.graphql.org/September2025/#VariableDefinition)
/// in an [`OperationDefinition`].
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct VariableDefinition {
    pub description: Option<Node<str>>,
    pub name: Name,
    pub ty: Node<Type>,
    pub default_value: Option<Node<Value>>,
    pub directives: DirectiveList,
}

/// Type system AST for a reference to a GraphQL [_Type_](https://spec.graphql.org/September2025/#Type)
#[derive(Clone, Debug, Eq, PartialEq, Hash, serde::Serialize, serde::Deserialize)]
pub enum Type {
    /// A `Foo` reference to nullable named type
    Named(NamedType),

    /// A `Foo!` reference to non-null named type
    NonNullNamed(NamedType),

    /// A `[…]` reference to nullable list type.
    /// (The inner item type may or may not be nullable, or a nested list.)
    List(Box<Type>),

    /// A `[…]!` reference to non-null list type.
    /// (The inner item type may or may not be nullable, or a nested list.)
    NonNullList(Box<Type>),
}

/// Type system AST for a [_FieldDefinition_](https://spec.graphql.org/September2025/#FieldDefinition)
/// in an object type or interface type defintion or extension.
#[derive(Clone, Debug)]
pub struct FieldDefinition {
    pub description: Option<Node<str>>,
    pub name: Name,
    pub arguments: Vec<Node<InputValueDefinition>>,
    pub ty: Type,
    pub directives: DirectiveList,
}

impl PartialEq for FieldDefinition {
    fn eq(&self, other: &Self) -> bool {
        self.description == other.description
            && self.name == other.name
            && self.ty == other.ty
            && self.directives == other.directives
            && input_value_definitions_eq(&self.arguments, &other.arguments)
    }
}

impl Eq for FieldDefinition {}

impl std::hash::Hash for FieldDefinition {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.description.hash(state);
        self.name.hash(state);
        self.ty.hash(state);
        self.directives.hash(state);
        input_value_definitions_hash(&self.arguments, state);
    }
}

/// Type system AST for an
/// [_InputValueDefinition_](https://spec.graphql.org/September2025/#InputValueDefinition),
/// a input type field definition or an argument definition.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct InputValueDefinition {
    pub description: Option<Node<str>>,
    pub name: Name,
    pub ty: Node<Type>,
    pub default_value: Option<Node<Value>>,
    pub directives: DirectiveList,
}

/// Type system AST for an
/// [_EnumValueDefinition_](https://spec.graphql.org/September2025/#EnumValueDefinition)
/// in an enum type definition or extension.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct EnumValueDefinition {
    pub description: Option<Node<str>>,
    pub value: Name,
    pub directives: DirectiveList,
}

/// Executable AST for a [_Selection_](https://spec.graphql.org/September2025/#Selection)
/// in a selection set.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum Selection {
    Field(Node<Field>),
    FragmentSpread(Node<FragmentSpread>),
    InlineFragment(Node<InlineFragment>),
}

/// Executable AST for a [_Field_](https://spec.graphql.org/September2025/#Field) selection
/// in a selection set.
#[derive(Clone, Debug)]
pub struct Field {
    pub alias: Option<Name>,
    pub name: Name,
    pub arguments: Vec<Node<Argument>>,
    pub directives: DirectiveList,
    pub selection_set: Vec<Selection>,
}

impl PartialEq for Field {
    fn eq(&self, other: &Self) -> bool {
        self.alias == other.alias
            && self.name == other.name
            && self.directives == other.directives
            && self.selection_set == other.selection_set
            && arguments_eq(&self.arguments, &other.arguments)
    }
}

impl Eq for Field {}

impl std::hash::Hash for Field {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.alias.hash(state);
        self.name.hash(state);
        self.directives.hash(state);
        self.selection_set.hash(state);
        arguments_hash(&self.arguments, state);
    }
}

/// Executable AST for a
/// [_FragmentSpread_](https://spec.graphql.org/September2025/#FragmentSpread) selection
/// in a selection set.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct FragmentSpread {
    pub fragment_name: Name,
    pub directives: DirectiveList,
}

/// Executable AST for an
/// [_InlineFragment_](https://spec.graphql.org/September2025/#InlineFragment) selection
/// in a selection set.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct InlineFragment {
    pub type_condition: Option<NamedType>,
    pub directives: DirectiveList,
    pub selection_set: Vec<Selection>,
}

/// Executable AST for a literal GraphQL [_Value_](https://spec.graphql.org/September2025/#Value).
#[derive(Clone, Debug, Eq, PartialEq, Hash, serde::Serialize, serde::Deserialize)]
pub enum Value {
    /// A [_NullValue_](https://spec.graphql.org/September2025/#NullValue)
    Null,

    /// An [_EnumValue_](https://spec.graphql.org/September2025/#EnumValue)
    Enum(Name),

    /// A [_Variable_](https://spec.graphql.org/September2025/#Variable)
    Variable(Name),

    /// A [_StringValue_](https://spec.graphql.org/September2025/#StringValue)
    String(
        /// The [semantic Unicode text](https://spec.graphql.org/September2025/#sec-String-Value.Static-Semantics)
        /// that this value represents.
        String,
    ),

    /// A [_FloatValue_](https://spec.graphql.org/September2025/#FloatValue)
    Float(FloatValue),

    /// An [_IntValue_](https://spec.graphql.org/September2025/#IntValue)
    Int(IntValue),

    /// A [_BooleanValue_](https://spec.graphql.org/September2025/#BooleanValue)
    Boolean(bool),

    /// A [_ListValue_](https://spec.graphql.org/September2025/#ListValue)
    List(Vec<Node<Value>>),

    /// An [_ObjectValue_](https://spec.graphql.org/September2025/#ObjectValue)
    Object(Vec<(Name, Node<Value>)>),
}

/// An [_IntValue_](https://spec.graphql.org/September2025/#IntValue),
/// represented as a string in order not to lose range or precision.
#[derive(Clone, Eq, PartialEq, Hash)]
pub struct IntValue(String);

/// An [_FloatValue_](https://spec.graphql.org/September2025/#FloatValue),
/// represented as a string in order not to lose range or precision.
#[derive(Clone, Eq, PartialEq, Hash)]
pub struct FloatValue(String);

/// Error type of [`IntValue::try_to_f64`] an  [`FloatValue::try_to_f64`]
/// for conversions that overflow `f64` and would be “rounded” to infinity.
#[derive(Clone, Eq, PartialEq)]
#[non_exhaustive]
pub struct FloatOverflowError {}

/// Error type of [`Directive::argument_by_name`] and
/// [`Field::argument_by_name`][crate::executable::Field::argument_by_name]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArgumentByNameError {
    /// The directive is not definied in the schema
    UndefinedDirective,
    /// The directive or field definition does not define an argument with the requested name
    NoSuchArgument,
    /// The argument is required (does not define a default value and has non-null type)
    /// but not specified
    RequiredArgumentNotSpecified,
}

/// Order-independent equality for argument lists.
///
/// Uses multiset matching to handle duplicate names correctly in
/// invalid-but-parseable documents.
pub(crate) fn arguments_eq(a: &[Node<Argument>], b: &[Node<Argument>]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut matched = vec![false; b.len()];
    a.iter().all(|arg_a| {
        b.iter()
            .enumerate()
            .find(|(i, arg_b)| {
                !matched[*i] && arg_a.name == arg_b.name && arg_a.value == arg_b.value
            })
            .map(|(i, _)| {
                matched[i] = true;
                true
            })
            .unwrap_or(false)
    })
}

/// Order-independent hash for argument lists.
pub(crate) fn arguments_hash<H: std::hash::Hasher>(args: &[Node<Argument>], state: &mut H) {
    args.len().hash(state);
    let mut combined = 0u64;
    for arg in args {
        let mut h = std::hash::DefaultHasher::new();
        arg.name.hash(&mut h);
        arg.value.hash(&mut h);
        combined ^= std::hash::Hasher::finish(&h);
    }
    combined.hash(state);
}

/// Order-independent equality for input value definition lists.
fn input_value_definitions_eq(
    a: &[Node<InputValueDefinition>],
    b: &[Node<InputValueDefinition>],
) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut matched = vec![false; b.len()];
    a.iter().all(|def_a| {
        b.iter()
            .enumerate()
            .find(|(i, def_b)| {
                !matched[*i]
                    && def_a.name == def_b.name
                    && def_a.ty == def_b.ty
                    && def_a.default_value == def_b.default_value
                    && def_a.directives == def_b.directives
            })
            .map(|(i, _)| {
                matched[i] = true;
                true
            })
            .unwrap_or(false)
    })
}

/// Order-independent hash for input value definition lists.
fn input_value_definitions_hash<H: std::hash::Hasher>(
    defs: &[Node<InputValueDefinition>],
    state: &mut H,
) {
    defs.len().hash(state);
    let mut combined = 0u64;
    for def in defs {
        let mut h = std::hash::DefaultHasher::new();
        def.name.hash(&mut h);
        def.ty.hash(&mut h);
        def.default_value.hash(&mut h);
        def.directives.hash(&mut h);
        combined ^= std::hash::Hasher::finish(&h);
    }
    combined.hash(state);
}
