use super::*;
use crate::hir::HirNodeLocation;
use std::fmt;

macro_rules! directive_by_name_method {
    () => {
        /// Returns the first directive with the given name, if any.
        ///
        /// This method is best for non-repeatable directives. For repeatable directives,
        /// see [`directives_by_name`][Self::directives_by_name] (plural)
        pub fn directive_by_name(&self, name: &str) -> Option<&Node<Directive>> {
            self.directives_by_name(name).next()
        }
    };
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
        ) -> impl Iterator<Item = &'def Node<Directive>> + 'name {
            directives_by_name(&self.directives, name)
        }

        directive_by_name_method!();
    };
}

fn directives_by_name<'def: 'name, 'name>(
    directives: &'def [Node<Directive>],
    name: &'name str,
) -> impl Iterator<Item = &'def Node<Directive>> + 'name {
    directives.iter().filter(move |dir| dir.name == name)
}

impl Document {
    /// Create an empty document
    pub fn new() -> Self {
        Self {
            definitions: Vec::new(),
        }
    }

    /// Return a new configurable parser
    pub fn parser() -> Parser {
        Parser::default()
    }

    /// Parse `input` with the default configuration
    pub fn parse(input: &str) -> ParseResult {
        Self::parser().parse(input)
    }

    serialize_method!();
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
    /// Returns true if this is an executable definition (operation or fragment).
    pub fn is_executable_definition(&self) -> bool {
        matches!(
            self,
            Self::OperationDefinition(_) | Self::FragmentDefinition(_)
        )
    }

    /// Returns true if this is an extension of another definition.
    pub fn is_extension_definition(&self) -> bool {
        matches!(
            self,
            Self::SchemaExtension(_)
                | Self::ScalarTypeExtension(_)
                | Self::ObjectTypeExtension(_)
                | Self::InterfaceTypeExtension(_)
                | Self::UnionTypeExtension(_)
                | Self::EnumTypeExtension(_)
                | Self::InputObjectTypeExtension(_)
        )
    }

    pub fn kind(&self) -> &'static str {
        match self {
            Self::OperationDefinition(_) => "OperationDefinition",
            Self::FragmentDefinition(_) => "FragmentDefinition",
            Self::DirectiveDefinition(_) => "DirectiveDefinition",
            Self::ScalarTypeDefinition(_) => "ScalarTypeDefinition",
            Self::ObjectTypeDefinition(_) => "ObjectTypeDefinition",
            Self::InterfaceTypeDefinition(_) => "InterfaceTypeDefinition",
            Self::UnionTypeDefinition(_) => "UnionTypeDefinition",
            Self::EnumTypeDefinition(_) => "EnumTypeDefinition",
            Self::InputObjectTypeDefinition(_) => "InputObjectTypeDefinition",
            Self::SchemaDefinition(_) => "SchemaDefinition",
            Self::SchemaExtension(_) => "SchemaExtension",
            Self::ScalarTypeExtension(_) => "ScalarTypeExtension",
            Self::ObjectTypeExtension(_) => "ObjectTypeExtension",
            Self::InterfaceTypeExtension(_) => "InterfaceTypeExtension",
            Self::UnionTypeExtension(_) => "UnionTypeExtension",
            Self::EnumTypeExtension(_) => "EnumTypeExtension",
            Self::InputObjectTypeExtension(_) => "InputObjectTypeExtension",
        }
    }

    pub fn location(&self) -> Option<&HirNodeLocation> {
        match self {
            Self::OperationDefinition(def) => def.location(),
            Self::FragmentDefinition(def) => def.location(),
            Self::DirectiveDefinition(def) => def.location(),
            Self::SchemaDefinition(def) => def.location(),
            Self::ScalarTypeDefinition(def) => def.location(),
            Self::ObjectTypeDefinition(def) => def.location(),
            Self::InterfaceTypeDefinition(def) => def.location(),
            Self::UnionTypeDefinition(def) => def.location(),
            Self::EnumTypeDefinition(def) => def.location(),
            Self::InputObjectTypeDefinition(def) => def.location(),
            Self::SchemaExtension(def) => def.location(),
            Self::ScalarTypeExtension(def) => def.location(),
            Self::ObjectTypeExtension(def) => def.location(),
            Self::InterfaceTypeExtension(def) => def.location(),
            Self::UnionTypeExtension(def) => def.location(),
            Self::EnumTypeExtension(def) => def.location(),
            Self::InputObjectTypeExtension(def) => def.location(),
        }
    }

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
            .find(|(arg_name, _value)| *arg_name == name)
            .map(|(_name, value)| value)
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
        Type::Named(Name::new_synthetic(name))
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

    /// Returns this type made nullable, if it isn’t already.
    pub fn nullable(self) -> Self {
        match self {
            Type::Named(_) => self,
            Type::List(_) => self,
            Type::NonNullNamed(name) => Type::Named(name),
            Type::NonNullList(inner) => Type::List(inner),
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

    pub fn is_non_null(&self) -> bool {
        matches!(self, Type::NonNullNamed(_) | Type::NonNullList(_))
    }

    /// Can a value of this type be used when the `target` type is expected?
    ///
    /// Implementation of spec function `AreTypesCompatible()`.
    pub fn is_assignable_to(&self, target: &Self) -> bool {
        match (target, self) {
            // Can't assign a nullable type to a non-nullable type.
            (Type::NonNullNamed(_) | Type::NonNullList(_), Type::Named(_) | Type::List(_)) => false,
            // Can't assign a list type to a non-list type.
            (Type::Named(_) | Type::NonNullNamed(_), Type::List(_) | Type::NonNullList(_)) => false,
            // Can't assign a non-list type to a list type.
            (Type::List(_) | Type::NonNullList(_), Type::Named(_) | Type::NonNullNamed(_)) => false,
            // Non-null named types can be assigned if they are the same.
            (Type::NonNullNamed(left), Type::NonNullNamed(right)) => left == right,
            // Non-null list types can be assigned if their inner types are compatible.
            (Type::NonNullList(left), Type::NonNullList(right)) => right.is_assignable_to(left),
            // Both nullable and non-nullable named types can be assigned to a nullable type of the
            // same name.
            (Type::Named(left), Type::Named(right) | Type::NonNullNamed(right)) => left == right,
            // Nullable and non-nullable lists can be assigned to a matching nullable list type.
            (Type::List(left), Type::List(right) | Type::NonNullList(right)) => {
                right.is_assignable_to(left)
            }
        }
    }

    serialize_method!();
}

impl FieldDefinition {
    directive_methods!();
    serialize_method!();
}

impl InputValueDefinition {
    pub fn is_required(&self) -> bool {
        matches!(self.ty, Type::NonNullNamed(_) | Type::NonNullList(_))
    }

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

    pub fn as_str(&self) -> Option<&NodeStr> {
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

    pub fn kind(&self) -> &'static str {
        match self {
            Value::Null => "Null",
            Value::Enum(_) => "Enum",
            Value::Variable(_) => "Variable",
            Value::String(_) => "String",
            Value::Float(_) => "Float",
            Value::Int(_) => "Int",
            Value::BigInt(_) => "BigInt",
            Value::Boolean(_) => "Boolean",
            Value::List(_) => "List",
            Value::Object(_) => "Object",
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

impl From<bool> for Value {
    fn from(value: bool) -> Self {
        Value::Boolean(value)
    }
}

impl From<()> for Node<Value> {
    fn from(value: ()) -> Self {
        Node::new_synthetic(value.into())
    }
}

impl From<ordered_float::OrderedFloat<f64>> for Node<Value> {
    fn from(value: ordered_float::OrderedFloat<f64>) -> Self {
        Node::new_synthetic(value.into())
    }
}

impl From<f64> for Node<Value> {
    fn from(value: f64) -> Self {
        Node::new_synthetic(value.into())
    }
}

impl From<i32> for Node<Value> {
    fn from(value: i32) -> Self {
        Node::new_synthetic(value.into())
    }
}

impl From<bool> for Node<Value> {
    fn from(value: bool) -> Self {
        Node::new_synthetic(value.into())
    }
}
