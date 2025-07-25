use crate::response::JsonMap;
use serde_json_bytes::Value as JsonValue;

/// A concrete GraphQL object whose fields can be resolved during execution.
pub(crate) trait ObjectValue {
    /// Returns the name of the concrete object type this resolver represents
    ///
    /// That name expected to be that of an object type defined in the schema.
    /// This is called when the schema indicates an abstract (interface or union) type.
    fn type_name(&self) -> &str;

    /// Resolves a concrete field of this object with the given arguments
    ///
    /// The resolved value is expected to match the type of the corresponding field definition
    /// in the schema.
    ///
    /// This is _not_ called for [introspection](https://spec.graphql.org/draft/#sec-Introspection)
    /// meta-fields `__typename`, `__type`, or `__schema`: those are handled separately.
    fn resolve_field<'a>(
        &'a self,
        field_name: &'a str,
        arguments: &'a JsonMap,
    ) -> Result<ResolvedValue<'a>, ResolveError>;
}

pub(crate) struct ResolveError {
    pub(crate) message: String,
}

impl ResolveError {
    pub(crate) fn unknown_field(field_name: &str, object: &dyn ObjectValue) -> Self {
        Self {
            message: format!(
                "unexpected field name: {field_name} in type {}",
                object.type_name()
            ),
        }
    }
}

/// The value of a resolved field
pub(crate) enum ResolvedValue<'a> {
    /// * JSON null represents GraphQL null
    /// * A GraphQL enum value is represented as a JSON string
    /// * GraphQL built-in scalars are coerced according to their respective *Result Coercion* spec
    /// * For custom scalars, any JSON value is passed through as-is (including array or object)
    Leaf(JsonValue),

    /// Expected where the GraphQL type is an object, interface, or union type
    Object(Box<dyn ObjectValue + 'a>),

    /// Expected for GraphQL list types
    List(Box<dyn Iterator<Item = Result<ResolvedValue<'a>, ResolveError>> + 'a>),
}

impl<'a> ResolvedValue<'a> {
    /// Construct a null leaf resolved value
    pub(crate) fn null() -> Self {
        Self::Leaf(JsonValue::Null)
    }

    /// Construct a leaf resolved value from something that is convertible to JSON
    pub(crate) fn leaf(json: impl Into<JsonValue>) -> Self {
        Self::Leaf(json.into())
    }

    /// Construct an object resolved value
    pub(crate) fn object(resolver: impl ObjectValue + 'a) -> Self {
        Self::Object(Box::new(resolver))
    }

    /// Construct an object resolved value or null
    pub(crate) fn opt_object(opt_resolver: Option<impl ObjectValue + 'a>) -> Self {
        match opt_resolver {
            Some(resolver) => Self::Object(Box::new(resolver)),
            None => Self::null(),
        }
    }

    /// Construct a list resolved value from an iterator
    ///
    /// If errors can happen during iteration,
    /// construct the [`ResolvedValue::List`] enum variant directly instead.
    pub(crate) fn list<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = Self>,
        I::IntoIter: 'a,
    {
        Self::List(Box::new(iter.into_iter().map(Ok)))
    }
}
