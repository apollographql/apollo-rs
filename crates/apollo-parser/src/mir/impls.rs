//! Inherent method implementations for MIR types

use super::*;

macro_rules! directive_methods {
    () => {
        /// Returns an iterator of directives with the given name.
        ///
        /// This method is best for repeatable directives. For non-repeatable directives,
        /// see [`directive_by_name`][Self::directive_by_name] (singular)
        pub fn directives_by_name<'def: 'name, 'name>(
            &'def self,
            name: &'name str,
        ) -> impl Iterator<Item = &'def Harc<Ranged<Directive>>> + 'name {
            self.directives.iter().filter(move |dir| dir.name == name)
        }

        /// Returns the first directive with the given name, if any.
        ///
        /// This method is best for non-repeatable directives. For repeatable directives,
        /// see [`directives_by_name`][Self::directives_by_name] (plural)
        pub fn directive_by_name(&self, name: &str) -> Option<&Harc<Ranged<Directive>>> {
            self.directives_by_name(name).next()
        }
    };
}

impl Document {}

impl Definition {}

impl OperationDefinition {
    directive_methods!();
}

impl FragmentDefinition {
    directive_methods!();
}

impl DirectiveDefinition {}

impl SchemaDefinition {
    directive_methods!();
}

impl ScalarTypeDefinition {
    directive_methods!();
}

impl ObjectTypeDefinition {
    directive_methods!();
}

impl InterfaceTypeDefinition {
    directive_methods!();
}

impl UnionTypeDefinition {
    directive_methods!();
}

impl EnumTypeDefinition {
    directive_methods!();
}

impl InputObjectTypeDefinition {
    directive_methods!();
}

impl SchemaExtension {
    directive_methods!();
}

impl ScalarTypeExtension {
    directive_methods!();
}

impl ObjectTypeExtension {
    directive_methods!();
}

impl InterfaceTypeExtension {
    directive_methods!();
}

impl UnionTypeExtension {
    directive_methods!();
}

impl EnumTypeExtension {
    directive_methods!();
}

impl InputObjectTypeExtension {
    directive_methods!();
}

impl Directive {
    pub fn argument_by_name(&self, name: &str) -> Option<&Value> {
        self.arguments
            .iter()
            .find(|(arg_name, _value)| *arg_name == name)
            .map(|(_name, value)| value)
    }
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
}

impl Type {
    /// Returns a new `Type::Named`, with string type conversion including from `&str`.
    pub fn new_named(name: impl Into<BowString>) -> Self {
        Type::Named(name.into())
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
}

impl FieldDefinition {
    directive_methods!();
}

impl InputValueDefinition {
    directive_methods!();
}

impl EnumValueDefinition {
    directive_methods!();
}

impl Selection {}

impl Field {
    directive_methods!();
}

impl FragmentSpread {
    directive_methods!();
}

impl InlineFragment {
    directive_methods!();
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

    pub fn as_list(&self) -> Option<&[Harc<Ranged<Value>>]> {
        if let Value::List(value) = self {
            Some(value)
        } else {
            None
        }
    }

    pub fn as_object(&self) -> Option<&[(Name, Harc<Ranged<Value>>)]> {
        if let Value::Object(value) = self {
            Some(value)
        } else {
            None
        }
    }
}
