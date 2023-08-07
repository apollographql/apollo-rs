//! # Middle-level Intermediate Representation (MIR)
//!
//! A data structure for documents matching the GraphQL grammar.
//! Serializing it should produce a string that can be re-parsed losslessly
//! and without syntax errors. (Although it may have validation errors.)
//!
//! The top-level type is [`Document`].
//!
//! ## Ownership and mutability
//!
//! MIR types are thread-safe: they implement [`Send`] and [`Sync`].
//!
//! [`triomphe::Arc`] is used for shared ownership.
//! To modify a value behind `Arc`, use [`Arc::make_mut`] to get a mutable reference.
//! This will clone the value if there were other `Arc`s pointing to it,
//! leaving them unmodified (copy-on-write semantics).
//!
//! ## Parsing
//!
//! After parsing a string input with [`Parser`],
//! MIR types can be converted from corresponding [AST types][crate::ast]
//! with either [`TryFrom`] or (for [`Document`]) [`From`].
//! When a syntax error causes a component not to be representable in MIR,
//! that component is silently skipped.
//! Where that is not possible, [`TryFrom`] returns an error.
//! Callers are expected to check [`SyntaxTree::errors`] for syntax errors.
//!
//! ## Serialization
//!
//! MIR types implement the [`Display`] and [`ToString`] traits,
//! serializing to GraphQL syntax with the default configuration (two space indentation).
//! Their `serialize` method returns a builder whose methods set configuration.
//! The builder similarly implements [`Display`] and [`ToString`].
//!
//! ## Example
//!
//! ```rust
//! use apollo_parser::Parser;
//!
//! let input = "{
//!     spline {
//!         reticulation
//!     }
//! }";
//! let parser = Parser::new(input);
//! let ast = parser.parse();
//! assert_eq!(0, ast.errors().len());
//! let mir = ast.into_mir();
//! assert_eq!(mir.serialize().no_indent().to_string(), "query { spline { reticulation } }");
//! ```

#[cfg(doc)]
use crate::Parser;
#[cfg(doc)]
use crate::SyntaxTree;
#[cfg(doc)]
use std::fmt::Display;

use crate::bowstring::BowString;

mod from_ast;
mod impls;
mod ref_;
mod serialize;

pub use self::ref_::Ref;
pub use self::serialize::Serialize;

// TODO: is it worth having `ExecutableDocument` and `TypeSystemDocument` Rust structs
// with Rust enums that can only represent the corresponding definitions?
#[derive(Clone, Debug, PartialEq, Hash)]
pub struct Document {
    pub definitions: Vec<Definition>,
}

const _: () = {
    const fn assert_send<T: Send>() {}
    const fn assert_sync<T: Sync>() {}
    assert_send::<Document>();
    assert_sync::<Document>();
};

/// An identifier
pub type Name = BowString;

/// Refers to the name of a GraphQL type defined elsewhere
pub type NamedType = Name;

#[derive(Clone, Debug, PartialEq, Hash)]
pub enum Definition {
    OperationDefinition(Ref<OperationDefinition>),
    FragmentDefinition(Ref<FragmentDefinition>),
    DirectiveDefinition(Ref<DirectiveDefinition>),
    SchemaDefinition(Ref<SchemaDefinition>),
    ScalarTypeDefinition(Ref<ScalarTypeDefinition>),
    ObjectTypeDefinition(Ref<ObjectTypeDefinition>),
    InterfaceTypeDefinition(Ref<InterfaceTypeDefinition>),
    UnionTypeDefinition(Ref<UnionTypeDefinition>),
    EnumTypeDefinition(Ref<EnumTypeDefinition>),
    InputObjectTypeDefinition(Ref<InputObjectTypeDefinition>),
    SchemaExtension(Ref<SchemaExtension>),
    ScalarTypeExtension(Ref<ScalarTypeExtension>),
    ObjectTypeExtension(Ref<ObjectTypeExtension>),
    InterfaceTypeExtension(Ref<InterfaceTypeExtension>),
    UnionTypeExtension(Ref<UnionTypeExtension>),
    EnumTypeExtension(Ref<EnumTypeExtension>),
    InputObjectTypeExtension(Ref<InputObjectTypeExtension>),
}

#[derive(Clone, Debug, PartialEq, Hash)]
pub struct OperationDefinition {
    pub operation_type: OperationType,
    pub name: Option<Name>,
    pub variables: Vec<Ref<VariableDefinition>>,
    pub directives: Vec<Ref<Directive>>,
    pub selection_set: Vec<Selection>,
}

#[derive(Clone, Debug, PartialEq, Hash)]
pub struct FragmentDefinition {
    pub name: Name,
    pub type_condition: NamedType,
    pub directives: Vec<Ref<Directive>>,
    pub selection_set: Vec<Selection>,
}

#[derive(Clone, Debug, PartialEq, Hash)]
pub struct DirectiveDefinition {
    pub description: Option<BowString>,
    pub name: Name,
    pub arguments: Vec<Ref<InputValueDefinition>>,
    pub repeatable: bool,
    pub locations: Vec<DirectiveLocation>,
}

#[derive(Clone, Debug, PartialEq, Hash)]
pub struct SchemaDefinition {
    pub description: Option<BowString>,
    pub directives: Vec<Ref<Directive>>,
    pub root_operations: Vec<(OperationType, NamedType)>,
}

#[derive(Clone, Debug, PartialEq, Hash)]
pub struct ScalarTypeDefinition {
    pub description: Option<BowString>,
    pub name: Name,
    pub directives: Vec<Ref<Directive>>,
}

#[derive(Clone, Debug, PartialEq, Hash)]
pub struct ObjectTypeDefinition {
    pub description: Option<BowString>,
    pub name: Name,
    pub implements_interfaces: Vec<Name>,
    pub directives: Vec<Ref<Directive>>,
    pub fields: Vec<Ref<FieldDefinition>>,
}

#[derive(Clone, Debug, PartialEq, Hash)]
pub struct InterfaceTypeDefinition {
    pub description: Option<BowString>,
    pub name: Name,
    pub implements_interfaces: Vec<Name>,
    pub directives: Vec<Ref<Directive>>,
    pub fields: Vec<Ref<FieldDefinition>>,
}

#[derive(Clone, Debug, PartialEq, Hash)]
pub struct UnionTypeDefinition {
    pub description: Option<BowString>,
    pub name: Name,
    pub directives: Vec<Ref<Directive>>,
    pub members: Vec<NamedType>,
}

#[derive(Clone, Debug, PartialEq, Hash)]
pub struct EnumTypeDefinition {
    pub description: Option<BowString>,
    pub name: Name,
    pub directives: Vec<Ref<Directive>>,
    pub values: Vec<Ref<EnumValueDefinition>>,
}

#[derive(Clone, Debug, PartialEq, Hash)]
pub struct InputObjectTypeDefinition {
    pub description: Option<BowString>,
    pub name: Name,
    pub directives: Vec<Ref<Directive>>,
    pub fields: Vec<Ref<InputValueDefinition>>,
}

#[derive(Clone, Debug, PartialEq, Hash)]
pub struct SchemaExtension {
    pub directives: Vec<Ref<Directive>>,
    pub root_operations: Vec<(OperationType, NamedType)>,
}

#[derive(Clone, Debug, PartialEq, Hash)]
pub struct ScalarTypeExtension {
    pub name: Name,
    pub directives: Vec<Ref<Directive>>,
}

#[derive(Clone, Debug, PartialEq, Hash)]
pub struct ObjectTypeExtension {
    pub name: Name,
    pub implements_interfaces: Vec<Name>,
    pub directives: Vec<Ref<Directive>>,
    pub fields: Vec<Ref<FieldDefinition>>,
}

#[derive(Clone, Debug, PartialEq, Hash)]
pub struct InterfaceTypeExtension {
    pub name: Name,
    pub implements_interfaces: Vec<Name>,
    pub directives: Vec<Ref<Directive>>,
    pub fields: Vec<Ref<FieldDefinition>>,
}

#[derive(Clone, Debug, PartialEq, Hash)]
pub struct UnionTypeExtension {
    pub name: Name,
    pub directives: Vec<Ref<Directive>>,
    pub members: Vec<NamedType>,
}

#[derive(Clone, Debug, PartialEq, Hash)]
pub struct EnumTypeExtension {
    pub name: Name,
    pub directives: Vec<Ref<Directive>>,
    pub values: Vec<Ref<EnumValueDefinition>>,
}

#[derive(Clone, Debug, PartialEq, Hash)]
pub struct InputObjectTypeExtension {
    pub name: Name,
    pub directives: Vec<Ref<Directive>>,
    pub fields: Vec<Ref<InputValueDefinition>>,
}

#[derive(Clone, Debug, PartialEq, Hash)]
pub struct Directive {
    pub name: Name,
    pub arguments: Vec<(Name, Value)>,
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum OperationType {
    Query,
    Mutation,
    Subscription,
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
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

#[derive(Clone, Debug, PartialEq, Hash)]
pub struct VariableDefinition {
    pub name: Name,
    pub ty: Type,
    pub default_value: Option<Value>,
    pub directives: Vec<Ref<Directive>>,
}

// TODO: is it worth making memory-compact representation?
// Could be a `NamedType` with a https://crates.io/crates/smallbitvec
// whose length is the list nesting depth + 1,
// and whose bits represents whether each nested level is non-null.
#[derive(Clone, Debug, PartialEq, Hash)]
pub enum Type {
    Named(NamedType),
    NonNullNamed(NamedType),
    List(Box<Type>),
    NonNullList(Box<Type>),
}

#[derive(Clone, Debug, PartialEq, Hash)]
pub struct FieldDefinition {
    pub description: Option<BowString>,
    pub name: Name,
    pub arguments: Vec<Ref<InputValueDefinition>>,
    pub ty: Type,
    pub directives: Vec<Ref<Directive>>,
}

#[derive(Clone, Debug, PartialEq, Hash)]
pub struct InputValueDefinition {
    pub description: Option<BowString>,
    pub name: Name,
    pub ty: Type,
    pub default_value: Option<Value>,
    pub directives: Vec<Ref<Directive>>,
}

#[derive(Clone, Debug, PartialEq, Hash)]
pub struct EnumValueDefinition {
    pub description: Option<BowString>,
    pub value: Name,
    pub directives: Vec<Ref<Directive>>,
}

#[derive(Clone, Debug, PartialEq, Hash)]
pub enum Selection {
    Field(Ref<Field>),
    FragmentSpread(Ref<FragmentSpread>),
    InlineFragment(Ref<InlineFragment>),
}

#[derive(Clone, Debug, PartialEq, Hash)]
pub struct Field {
    pub alias: Option<Name>,
    pub name: Name,
    pub arguments: Vec<(Name, Value)>,
    pub directives: Vec<Ref<Directive>>,
    pub selection_set: Vec<Selection>,
}

#[derive(Clone, Debug, PartialEq, Hash)]
pub struct FragmentSpread {
    pub fragment_name: Name,
    pub directives: Vec<Ref<Directive>>,
}

#[derive(Clone, Debug, PartialEq, Hash)]
pub struct InlineFragment {
    pub type_condition: Option<NamedType>,
    pub directives: Vec<Ref<Directive>>,
    pub selection_set: Vec<Selection>,
}

#[derive(Clone, Debug, PartialEq, Hash)]
pub enum Value {
    Null,
    Enum(Name),
    Variable(Name),
    String(
        /// The value after escape sequences are resolved
        BowString,
    ),
    Float(ordered_float::OrderedFloat<f64>),
    Int(i32),
    /// Integer syntax (without a decimal point) but overflows `i32`.
    /// Valid in contexts where the expected GraphQL type is Float.
    BigInt(
        /// Must only contain ASCII decimal digits
        BowString,
    ),
    Boolean(bool),
    List(Vec<Ref<Value>>),           // TODO: is structural sharing useful here?
    Object(Vec<(Name, Ref<Value>)>), // TODO: is structural sharing useful here?
}
