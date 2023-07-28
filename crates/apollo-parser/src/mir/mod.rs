//! # Middle-level Intermediate Representation
//!
//! A data structure for documents matching the GraphQL grammar.
//! Serializing it should produce a string that can be re-parsed losslessly
//! and without syntax errors. (Although it may have validation errors.)

use crate::bowstring::BowString;
use triomphe::Arc;

mod from_ast;

// TODO: is it worth having `ExecutableDocument` and `TypeSystemDocument` Rust structs
// with Rust enums that can only represent the corresponding definitions?
#[derive(Clone, Debug)]
pub struct Document {
    pub definitions: Vec<Arc<Definition>>,
}

/// An identifier
pub type Name = BowString;

/// Refers to the name of a GraphQL type defined elsewhere
pub type NamedType = Name;

#[derive(Clone, Debug)]
pub enum Definition {
    OperationDefinition(OperationDefinition),
    FragmentDefinition(FragmentDefinition),
    DirectiveDefinition(DirectiveDefinition),
    SchemaDefinition(SchemaDefinition),
    ScalarTypeDefinition(ScalarTypeDefinition),
    ObjectTypeDefinition(ObjectTypeDefinition),
    InterfaceTypeDefinition(InterfaceTypeDefinition),
    UnionTypeDefinition(UnionTypeDefinition),
    EnumTypeDefinition(EnumTypeDefinition),
    InputObjectTypeDefinition(InputObjectTypeDefinition),
    SchemaExtension(SchemaExtension),
    ScalarTypeExtension(ScalarTypeExtension),
    ObjectTypeExtension(ObjectTypeExtension),
    InterfaceTypeExtension(InterfaceTypeExtension),
    UnionTypeExtension(UnionTypeExtension),
    EnumTypeExtension(EnumTypeExtension),
    InputObjectTypeExtension(InputObjectTypeExtension),
}

#[derive(Clone, Debug)]
pub struct OperationDefinition {
    pub operation_type: OperationType,
    pub name: Option<Name>,
    pub variables: Vec<Arc<VariableDefinition>>,
    pub directives: Vec<Arc<Directive>>,
    pub selection_set: Vec<Arc<Selection>>,
}

#[derive(Clone, Debug)]
pub struct FragmentDefinition {
    pub name: Name,
    pub type_condition: Option<NamedType>,
    pub directives: Vec<Arc<Directive>>,
    pub selection_set: Vec<Arc<Selection>>,
}

#[derive(Clone, Debug)]
pub struct DirectiveDefinition {
    pub description: Option<BowString>,
    pub name: Name,
    pub arguments: Vec<Arc<InputValueDefinition>>,
    pub repeatable: bool,
    pub locations: Vec<DirectiveLocation>,
}

#[derive(Clone, Debug)]
pub struct SchemaDefinition {
    pub description: Option<BowString>,
    pub directives: Vec<Arc<Directive>>,
    pub root_operations: Vec<(OperationType, NamedType)>,
}

#[derive(Clone, Debug)]
pub struct ScalarTypeDefinition {
    pub description: Option<BowString>,
    pub name: Name,
    pub directives: Vec<Arc<Directive>>,
}

#[derive(Clone, Debug)]
pub struct ObjectTypeDefinition {
    pub description: Option<BowString>,
    pub name: Name,
    pub implements_interfaces: Vec<Name>,
    pub directives: Vec<Arc<Directive>>,
    pub fields: Vec<Arc<FieldDefinition>>,
}

#[derive(Clone, Debug)]
pub struct InterfaceTypeDefinition {
    pub description: Option<BowString>,
    pub name: Name,
    pub implements_interfaces: Vec<Name>,
    pub directives: Vec<Arc<Directive>>,
    pub fields: Vec<Arc<FieldDefinition>>,
}

#[derive(Clone, Debug)]
pub struct UnionTypeDefinition {
    pub description: Option<BowString>,
    pub name: Name,
    pub directives: Vec<Arc<Directive>>,
    pub members: Vec<NamedType>,
}

#[derive(Clone, Debug)]
pub struct EnumTypeDefinition {
    pub description: Option<BowString>,
    pub name: Name,
    pub directives: Vec<Arc<Directive>>,
    pub values: Vec<Arc<EnumValueDefinition>>,
}

#[derive(Clone, Debug)]
pub struct InputObjectTypeDefinition {
    pub description: Option<BowString>,
    pub name: Name,
    pub directives: Vec<Arc<Directive>>,
    pub fields: Vec<Arc<InputValueDefinition>>,
}

#[derive(Clone, Debug)]
pub struct SchemaExtension {
    pub directives: Vec<Arc<Directive>>,
    pub root_operations: Vec<(OperationType, NamedType)>,
}

#[derive(Clone, Debug)]
pub struct ScalarTypeExtension {
    pub name: Name,
    pub directives: Vec<Arc<Directive>>,
}

#[derive(Clone, Debug)]
pub struct ObjectTypeExtension {
    pub name: Name,
    pub implements_interfaces: Vec<Name>,
    pub directives: Vec<Arc<Directive>>,
    pub fields: Vec<Arc<FieldDefinition>>,
}

#[derive(Clone, Debug)]
pub struct InterfaceTypeExtension {
    pub name: Name,
    pub implements_interfaces: Vec<Name>,
    pub directives: Vec<Arc<Directive>>,
    pub fields: Vec<Arc<FieldDefinition>>,
}

#[derive(Clone, Debug)]
pub struct UnionTypeExtension {
    pub name: Name,
    pub directives: Vec<Arc<Directive>>,
    pub members: Vec<NamedType>,
}

#[derive(Clone, Debug)]
pub struct EnumTypeExtension {
    pub name: Name,
    pub directives: Vec<Arc<Directive>>,
    pub values: Vec<Arc<EnumValueDefinition>>,
}

#[derive(Clone, Debug)]
pub struct InputObjectTypeExtension {
    pub name: Name,
    pub directives: Vec<Arc<Directive>>,
    pub fields: Vec<Arc<InputValueDefinition>>,
}

#[derive(Clone, Debug)]
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

impl DirectiveLocation {
    /// Get the name of this directive location as it would appear in GraphQL source code.
    pub fn name(self) -> &'static str {
        match self {
            DirectiveLocation::Query => "QUERY",
            DirectiveLocation::Mutation => "MUTATION",
            DirectiveLocation::Subscription => "SUBSCRIPTION",
            DirectiveLocation::Field => "FIELD",
            DirectiveLocation::FragmentDefinition => "FRAGMENT_DEFINITION",
            DirectiveLocation::FragmentSpread => "FRAGMENT_SPREAD",
            DirectiveLocation::InlineFragment => "INLINE_FRAGMENT",
            DirectiveLocation::VariableDefinition => "VARIABLE_DEFINITION",
            DirectiveLocation::Schema => "SCHEMA",
            DirectiveLocation::Scalar => "SCALAR",
            DirectiveLocation::Object => "OBJECT",
            DirectiveLocation::FieldDefinition => "FIELD_DEFINITION",
            DirectiveLocation::ArgumentDefinition => "ARGUMENT_DEFINITION",
            DirectiveLocation::Interface => "INTERFACE",
            DirectiveLocation::Union => "UNION",
            DirectiveLocation::Enum => "ENUM",
            DirectiveLocation::EnumValue => "ENUM_VALUE",
            DirectiveLocation::InputObject => "INPUT_OBJECT",
            DirectiveLocation::InputFieldDefinition => "INPUT_FIELD_DEFINITION",
        }
    }
}

impl std::fmt::Display for DirectiveLocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.name().fmt(f)
    }
}

#[derive(Clone, Debug)]
pub struct VariableDefinition {
    pub name: Name,
    pub ty: Type,
    pub default_value: Option<Value>,
    pub directives: Vec<Arc<Directive>>,
}

/// TODO: is it worth making memory-compact representation?
/// Could be a `NamedType` with a https://crates.io/crates/smallbitvec
/// whose length is the list nesting depth + 1,
/// and whose bits represents whether each nested level is non-null.
#[derive(Clone, Debug)]
pub enum Type {
    Named(NamedType),
    NonNullNamed(NamedType),
    List(Box<Type>),
    NonNullList(Box<Type>),
}

#[derive(Clone, Debug)]
pub struct FieldDefinition {
    pub description: Option<BowString>,
    pub name: Name,
    pub arguments: Vec<Arc<InputValueDefinition>>,
    pub ty: Type,
    pub directives: Vec<Arc<Directive>>,
}

#[derive(Clone, Debug)]
pub struct InputValueDefinition {
    pub description: Option<BowString>,
    pub name: Name,
    pub ty: Type,
    pub default_value: Option<Value>,
    pub directives: Vec<Arc<Directive>>,
}

#[derive(Clone, Debug)]
pub struct EnumValueDefinition {
    pub description: Option<BowString>,
    pub value: Name,
    pub directives: Vec<Arc<Directive>>,
}

#[derive(Clone, Debug)]
pub enum Selection {
    Field(Field),
    FragmentSpread(FragmentSpread),
    InlineFragment(InlineFragment),
}

#[derive(Clone, Debug)]
pub struct Field {
    pub alias: Option<Name>,
    pub name: Name,
    pub arguments: Vec<(Name, Value)>,
    pub directives: Vec<Arc<Directive>>,
    pub selection_set: Vec<Arc<Selection>>,
}

#[derive(Clone, Debug)]
pub struct FragmentSpread {
    pub fragment_name: Name,
    pub directives: Vec<Arc<Directive>>,
}

#[derive(Clone, Debug)]
pub struct InlineFragment {
    pub type_condition: Option<NamedType>,
    pub directives: Vec<Arc<Directive>>,
    pub selection_set: Vec<Arc<Selection>>,
}

#[derive(Clone, Debug)]
pub enum Value {
    NullValue,
    EnumValue(Name),
    Variable(Name),
    StringValue(
        /// The value after escape sequences are resolved
        String,
    ),
    FloatValue(f64),
    IntValue(i32),
    /// Integer syntax (without a decimal point) but overflows `i32`.
    /// Valid in contexts where the expected GraphQL type is Float.
    BigInt(
        /// Must only contain ASCII decimal digits
        String,
    ),
    BooleanValue(bool),
    ListValue(Vec<Arc<Value>>), // TODO: is structural sharing useful here?
    ObjectValue(Vec<(Name, Arc<Value>)>), // TODO: is structural sharing useful here?
}
