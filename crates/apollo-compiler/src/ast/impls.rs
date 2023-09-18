use super::*;
use crate::schema::SchemaBuilder;
use crate::ExecutableDocument;
use crate::ParseError;
use crate::Parser;
use crate::Schema;
use std::fmt;
use std::hash;
use std::path::Path;

pub(crate) fn directives_by_name<'def: 'name, 'name>(
    directives: &'def [Node<Directive>],
    name: &'name str,
) -> impl Iterator<Item = &'def Node<Directive>> + 'name {
    directives.iter().filter(move |dir| dir.name == name)
}

impl Document {
    /// Create an empty document
    pub fn new() -> Self {
        Self {
            source: None,
            definitions: Vec::new(),
        }
    }

    /// Return a new configurable parser
    pub fn parser() -> Parser {
        Parser::default()
    }

    /// Parse `input` with the default configuration
    ///
    /// `path` is the filesystem path (or arbitrary string) used in diagnostics
    /// to identify this source file to users.
    pub fn parse(source_text: impl Into<String>, path: impl AsRef<Path>) -> Self {
        Self::parser().parse_ast(source_text, path)
    }

    pub fn parse_errors(&self) -> &[ParseError] {
        if let Some((_id, source)) = &self.source {
            source.parse_errors()
        } else {
            &[]
        }
    }

    /// Build a schema with this AST document as its sole input.
    pub fn to_schema(&self) -> Schema {
        let mut builder = Schema::builder();
        let executable_definitions_are_errors = true;
        builder.add_ast_document(self, executable_definitions_are_errors);
        builder.build()
    }

    /// Add this AST document as an additional input to a schema builder.
    ///
    /// This can be used to build a schema from multiple documents or source files.
    pub fn to_schema_builder(&self, builder: &mut SchemaBuilder) {
        let executable_definitions_are_errors = true;
        builder.add_ast_document(self, executable_definitions_are_errors)
    }

    /// Build an executable document from this AST, with the given schema
    pub fn to_executable(&self, schema: &Schema) -> ExecutableDocument {
        let type_system_definitions_are_errors = true;
        crate::executable::from_ast::document_from_ast(
            schema,
            self,
            type_system_definitions_are_errors,
        )
    }

    /// Build a schema and executable document from this AST containing a mixture
    /// of type system definitions and executable definitions.
    /// This is mostly useful for unit tests.
    pub fn to_mixed(&self) -> (Schema, ExecutableDocument) {
        let executable_definitions_are_errors = false;
        let type_system_definitions_are_errors = false;
        let mut builder = Schema::builder();
        builder.add_ast_document(self, executable_definitions_are_errors);
        let schema = builder.build();
        let executable = crate::executable::from_ast::document_from_ast(
            &schema,
            self,
            type_system_definitions_are_errors,
        );
        (schema, executable)
    }

    serialize_method!();
}

/// `source` is ignored for comparison
impl PartialEq for Document {
    fn eq(&self, other: &Self) -> bool {
        self.definitions == other.definitions
    }
}

impl Eq for Document {}

impl hash::Hash for Document {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.definitions.hash(state);
    }
}

impl Default for Document {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for Document {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Skip two not-useful indentation levels
        for def in &self.definitions {
            def.fmt(f)?;
            f.write_str("\n")?;
        }
        Ok(())
    }
}

impl Definition {
    /// Returns an iterator of directives with the given name.
    ///
    /// This method is best for repeatable directives. For non-repeatable directives,
    /// see [`directive_by_name`][Self::directive_by_name] (singular)
    pub fn directives_by_name<'def: 'name, 'name>(
        &'def self,
        name: &'name str,
    ) -> impl Iterator<Item = &'def Node<Directive>> + 'name {
        match self {
            Self::OperationDefinition(def) => directives_by_name(&def.directives, name),
            Self::FragmentDefinition(def) => directives_by_name(&def.directives, name),
            Self::DirectiveDefinition(_) => directives_by_name(&[], name),
            Self::SchemaDefinition(def) => directives_by_name(&def.directives, name),
            Self::ScalarTypeDefinition(def) => directives_by_name(&def.directives, name),
            Self::ObjectTypeDefinition(def) => directives_by_name(&def.directives, name),
            Self::InterfaceTypeDefinition(def) => directives_by_name(&def.directives, name),
            Self::UnionTypeDefinition(def) => directives_by_name(&def.directives, name),
            Self::EnumTypeDefinition(def) => directives_by_name(&def.directives, name),
            Self::InputObjectTypeDefinition(def) => directives_by_name(&def.directives, name),
            Self::SchemaExtension(def) => directives_by_name(&def.directives, name),
            Self::ScalarTypeExtension(def) => directives_by_name(&def.directives, name),
            Self::ObjectTypeExtension(def) => directives_by_name(&def.directives, name),
            Self::InterfaceTypeExtension(def) => directives_by_name(&def.directives, name),
            Self::UnionTypeExtension(def) => directives_by_name(&def.directives, name),
            Self::EnumTypeExtension(def) => directives_by_name(&def.directives, name),
            Self::InputObjectTypeExtension(def) => directives_by_name(&def.directives, name),
        }
    }

    directive_by_name_method!();
    serialize_method!();
}

impl fmt::Debug for Definition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Skip the enum variant name as it’s redundant with the struct name in it
        match self {
            Self::OperationDefinition(def) => def.fmt(f),
            Self::FragmentDefinition(def) => def.fmt(f),
            Self::DirectiveDefinition(def) => def.fmt(f),
            Self::SchemaDefinition(def) => def.fmt(f),
            Self::ScalarTypeDefinition(def) => def.fmt(f),
            Self::ObjectTypeDefinition(def) => def.fmt(f),
            Self::InterfaceTypeDefinition(def) => def.fmt(f),
            Self::UnionTypeDefinition(def) => def.fmt(f),
            Self::EnumTypeDefinition(def) => def.fmt(f),
            Self::InputObjectTypeDefinition(def) => def.fmt(f),
            Self::SchemaExtension(def) => def.fmt(f),
            Self::ScalarTypeExtension(def) => def.fmt(f),
            Self::ObjectTypeExtension(def) => def.fmt(f),
            Self::InterfaceTypeExtension(def) => def.fmt(f),
            Self::UnionTypeExtension(def) => def.fmt(f),
            Self::EnumTypeExtension(def) => def.fmt(f),
            Self::InputObjectTypeExtension(def) => def.fmt(f),
        }
    }
}

impl OperationDefinition {
    directive_methods!();
    serialize_method!();
}

impl FragmentDefinition {
    directive_methods!();
    serialize_method!();
}

impl DirectiveDefinition {
    serialize_method!();
}

impl SchemaDefinition {
    directive_methods!();
    serialize_method!();
}

impl ScalarTypeDefinition {
    directive_methods!();
    serialize_method!();
}

impl ObjectTypeDefinition {
    directive_methods!();
    serialize_method!();
}

impl InterfaceTypeDefinition {
    directive_methods!();
    serialize_method!();
}

impl UnionTypeDefinition {
    directive_methods!();
    serialize_method!();
}

impl EnumTypeDefinition {
    directive_methods!();
    serialize_method!();
}

impl InputObjectTypeDefinition {
    directive_methods!();
    serialize_method!();
}

impl SchemaExtension {
    directive_methods!();
    serialize_method!();
}

impl ScalarTypeExtension {
    directive_methods!();
    serialize_method!();
}

impl ObjectTypeExtension {
    directive_methods!();
    serialize_method!();
}

impl InterfaceTypeExtension {
    directive_methods!();
    serialize_method!();
}

impl UnionTypeExtension {
    directive_methods!();
    serialize_method!();
}

impl EnumTypeExtension {
    directive_methods!();
    serialize_method!();
}

impl InputObjectTypeExtension {
    directive_methods!();
    serialize_method!();
}

impl Directive {
    pub fn argument_by_name(&self, name: &str) -> Option<&Node<Value>> {
        self.arguments
            .iter()
            .find_map(|arg| (arg.name == name).then_some(&arg.value))
    }

    serialize_method!();
}

impl OperationType {
    /// Get the name of this operation type as it would appear in GraphQL source code.
    pub fn name(self) -> &'static str {
        match self {
            OperationType::Query => "query",
            OperationType::Mutation => "mutation",
            OperationType::Subscription => "subscription",
        }
    }

    /// Get the default name of the object type for this operation type
    pub fn default_type_name(self) -> &'static str {
        match self {
            OperationType::Query => "Query",
            OperationType::Mutation => "Mutation",
            OperationType::Subscription => "Subscription",
        }
    }

    serialize_method!();
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

impl fmt::Debug for DirectiveLocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.name().fmt(f)
    }
}

impl From<OperationType> for DirectiveLocation {
    fn from(ty: OperationType) -> Self {
        match ty {
            OperationType::Query => DirectiveLocation::Query,
            OperationType::Mutation => DirectiveLocation::Mutation,
            OperationType::Subscription => DirectiveLocation::Subscription,
        }
    }
}

impl VariableDefinition {
    directive_methods!();
    serialize_method!();
}

impl Type {
    /// Returns a new `Type::Named` with with a synthetic `Name` (not parsed from a source file)
    pub fn new_named(name: &str) -> Self {
        Type::Named(Name::new(name))
    }

    /// Returns this type made non-null, if it isn’t already.
    pub fn non_null(self) -> Self {
        match self {
            Type::Named(name) => Type::NonNullNamed(name),
            Type::List(inner) => Type::NonNullList(inner),
            Type::NonNullNamed(_) => self,
            Type::NonNullList(_) => self,
        }
    }

    /// Returns a list type whose items are this type.
    pub fn list(self) -> Self {
        Type::List(Box::new(self))
    }

    /// Returns the inner named type, after unwrapping any non-null or list markers.
    pub fn inner_named_type(&self) -> &NamedType {
        match self {
            Type::Named(name) | Type::NonNullNamed(name) => name,
            Type::List(inner) | Type::NonNullList(inner) => inner.inner_named_type(),
        }
    }

    /// Returns whether this type is non-null
    pub fn is_non_null(&self) -> bool {
        matches!(self, Type::NonNullNamed(_) | Type::NonNullList(_))
    }

    /// Returns whether this type is a list, on a non-null list
    pub fn is_list(&self) -> bool {
        matches!(self, Type::List(_) | Type::NonNullList(_))
    }

    serialize_method!();
}

impl FieldDefinition {
    directive_methods!();
    serialize_method!();
}

impl InputValueDefinition {
    directive_methods!();
    serialize_method!();
}

impl EnumValueDefinition {
    directive_methods!();
    serialize_method!();
}

impl Selection {
    serialize_method!();
}

impl Field {
    directive_methods!();
    serialize_method!();
}

impl FragmentSpread {
    directive_methods!();
    serialize_method!();
}

impl InlineFragment {
    directive_methods!();
    serialize_method!();
}

impl Value {
    pub fn is_null(&self) -> bool {
        matches!(self, Value::Null)
    }

    pub fn as_enum(&self) -> Option<&Name> {
        if let Value::Enum(name) = self {
            Some(name)
        } else {
            None
        }
    }

    pub fn as_variable(&self) -> Option<&Name> {
        if let Value::Variable(name) = self {
            Some(name)
        } else {
            None
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        if let Value::String(value) = self {
            Some(value)
        } else {
            None
        }
    }

    pub fn as_node_str(&self) -> Option<&NodeStr> {
        if let Value::String(value) = self {
            Some(value)
        } else {
            None
        }
    }

    pub fn to_f64(&self) -> Option<f64> {
        match self {
            Value::Float(value) => Some(**value),
            Value::BigInt(value) => {
                if let Ok(f) = value.parse::<f64>() {
                    f.is_finite().then_some(f)
                } else {
                    // TODO: panic for invalid BigInt string value? (contains non-ASCII-digit characters)
                    None
                }
            }
            _ => None,
        }
    }

    pub fn to_i32(&self) -> Option<i32> {
        if let Value::Int(value) = *self {
            Some(value)
        } else {
            None
        }
    }

    pub fn to_bool(&self) -> Option<bool> {
        if let Value::Boolean(value) = *self {
            Some(value)
        } else {
            None
        }
    }

    pub fn as_list(&self) -> Option<&[Node<Value>]> {
        if let Value::List(value) = self {
            Some(value)
        } else {
            None
        }
    }

    pub fn as_object(&self) -> Option<&[(Name, Node<Value>)]> {
        if let Value::Object(value) = self {
            Some(value)
        } else {
            None
        }
    }

    serialize_method!();
}

impl From<Node<OperationDefinition>> for Definition {
    fn from(def: Node<OperationDefinition>) -> Self {
        Self::OperationDefinition(def)
    }
}

impl From<Node<FragmentDefinition>> for Definition {
    fn from(def: Node<FragmentDefinition>) -> Self {
        Self::FragmentDefinition(def)
    }
}

impl From<Node<DirectiveDefinition>> for Definition {
    fn from(def: Node<DirectiveDefinition>) -> Self {
        Self::DirectiveDefinition(def)
    }
}

impl From<Node<SchemaDefinition>> for Definition {
    fn from(def: Node<SchemaDefinition>) -> Self {
        Self::SchemaDefinition(def)
    }
}

impl From<Node<ScalarTypeDefinition>> for Definition {
    fn from(def: Node<ScalarTypeDefinition>) -> Self {
        Self::ScalarTypeDefinition(def)
    }
}

impl From<Node<ObjectTypeDefinition>> for Definition {
    fn from(def: Node<ObjectTypeDefinition>) -> Self {
        Self::ObjectTypeDefinition(def)
    }
}

impl From<Node<InterfaceTypeDefinition>> for Definition {
    fn from(def: Node<InterfaceTypeDefinition>) -> Self {
        Self::InterfaceTypeDefinition(def)
    }
}

impl From<Node<UnionTypeDefinition>> for Definition {
    fn from(def: Node<UnionTypeDefinition>) -> Self {
        Self::UnionTypeDefinition(def)
    }
}

impl From<Node<EnumTypeDefinition>> for Definition {
    fn from(def: Node<EnumTypeDefinition>) -> Self {
        Self::EnumTypeDefinition(def)
    }
}

impl From<Node<InputObjectTypeDefinition>> for Definition {
    fn from(def: Node<InputObjectTypeDefinition>) -> Self {
        Self::InputObjectTypeDefinition(def)
    }
}

impl From<Node<SchemaExtension>> for Definition {
    fn from(def: Node<SchemaExtension>) -> Self {
        Self::SchemaExtension(def)
    }
}

impl From<Node<ScalarTypeExtension>> for Definition {
    fn from(def: Node<ScalarTypeExtension>) -> Self {
        Self::ScalarTypeExtension(def)
    }
}

impl From<Node<ObjectTypeExtension>> for Definition {
    fn from(def: Node<ObjectTypeExtension>) -> Self {
        Self::ObjectTypeExtension(def)
    }
}

impl From<Node<InterfaceTypeExtension>> for Definition {
    fn from(def: Node<InterfaceTypeExtension>) -> Self {
        Self::InterfaceTypeExtension(def)
    }
}

impl From<Node<UnionTypeExtension>> for Definition {
    fn from(def: Node<UnionTypeExtension>) -> Self {
        Self::UnionTypeExtension(def)
    }
}

impl From<Node<EnumTypeExtension>> for Definition {
    fn from(def: Node<EnumTypeExtension>) -> Self {
        Self::EnumTypeExtension(def)
    }
}

impl From<Node<InputObjectTypeExtension>> for Definition {
    fn from(def: Node<InputObjectTypeExtension>) -> Self {
        Self::InputObjectTypeExtension(def)
    }
}

impl From<()> for Value {
    fn from(_value: ()) -> Self {
        Value::Null
    }
}

impl From<ordered_float::OrderedFloat<f64>> for Value {
    fn from(value: ordered_float::OrderedFloat<f64>) -> Self {
        Value::Float(value)
    }
}

impl From<f64> for Value {
    fn from(value: f64) -> Self {
        Value::Float(value.into())
    }
}

impl From<i32> for Value {
    fn from(value: i32) -> Self {
        Value::Int(value)
    }
}

impl From<&'_ str> for Value {
    fn from(value: &'_ str) -> Self {
        Value::String(value.into())
    }
}

impl From<&'_ String> for Value {
    fn from(value: &'_ String) -> Self {
        Value::String(value.into())
    }
}

impl From<String> for Value {
    fn from(value: String) -> Self {
        Value::String(value.into())
    }
}

impl From<bool> for Value {
    fn from(value: bool) -> Self {
        Value::Boolean(value)
    }
}

impl From<()> for Node<Value> {
    fn from(value: ()) -> Self {
        Node::new(value.into())
    }
}

impl From<ordered_float::OrderedFloat<f64>> for Node<Value> {
    fn from(value: ordered_float::OrderedFloat<f64>) -> Self {
        Node::new(value.into())
    }
}

impl From<f64> for Node<Value> {
    fn from(value: f64) -> Self {
        Node::new(value.into())
    }
}

impl From<i32> for Node<Value> {
    fn from(value: i32) -> Self {
        Node::new(value.into())
    }
}

impl From<&'_ str> for Node<Value> {
    fn from(value: &'_ str) -> Self {
        Node::new(value.into())
    }
}

impl From<&'_ String> for Node<Value> {
    fn from(value: &'_ String) -> Self {
        Node::new(value.into())
    }
}

impl From<String> for Node<Value> {
    fn from(value: String) -> Self {
        Node::new(value.into())
    }
}

impl From<bool> for Node<Value> {
    fn from(value: bool) -> Self {
        Node::new(value.into())
    }
}

impl<N: Into<Name>, V: Into<Node<Value>>> From<(N, V)> for Node<Argument> {
    fn from((name, value): (N, V)) -> Self {
        Node::new(Argument {
            name: name.into(),
            value: value.into(),
        })
    }
}
