use crate::response::JsonMap;
use serde_json_bytes::Value as JsonValue;

/// A GraphQL object whose fields can be resolved during execution
pub(crate) type ObjectValue<'a> = dyn Resolver + 'a;

/// Abstraction for implementing field resolvers. Used through [`ObjectValue`].
///
/// Use the [`impl_resolver!`][crate::impl_resolver] macro to implement this trait
/// with reduced boilerplate
pub(crate) trait Resolver {
    /// Returns the name of the concrete object type this resolver represents
    ///
    /// That name expected to be that of an object type defined in the schema.
    /// This is called when the schema indicates an abstract (interface or union) type.
    fn type_name(&self) -> &str;

    /// Resolves a field of this object with the given arguments
    ///
    /// The resolved is expected to match the type of the corresponding field definition
    /// in the schema.
    fn resolve_field<'a>(
        &'a self,
        field_name: &'a str,
        arguments: &'a JsonMap,
    ) -> Result<ResolvedValue<'a>, ResolverError>;

    /// Returns true if this field should be skipped,
    /// as if the corresponding selection has `@skip(if: true)`.
    ///
    /// This is used to exclude root concrete fields in [crate::introspection::partial_execute].
    fn skip_field(&self, _field_name: &str) -> bool {
        false
    }
}

pub(crate) struct ResolverError {
    pub(crate) message: String,
}

impl ResolverError {
    pub(crate) fn unknown_field(field_name: &str, object: &ObjectValue) -> Self {
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
    Object(Box<ObjectValue<'a>>),

    /// Expected for GraphQL list types
    List(Box<dyn Iterator<Item = Result<ResolvedValue<'a>, ResolverError>> + 'a>),
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

    /// Construct an object resolved value from the resolver for that object
    pub(crate) fn object(resolver: impl Resolver + 'a) -> Self {
        Self::Object(Box::new(resolver))
    }

    /// Construct an object resolved value or null, from an optional resolver
    pub(crate) fn opt_object(opt_resolver: Option<impl Resolver + 'a>) -> Self {
        match opt_resolver {
            Some(resolver) => Self::Object(Box::new(resolver)),
            None => Self::null(),
        }
    }

    /// Construct a list resolved value from an iterator
    pub(crate) fn list<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = Self>,
        I::IntoIter: 'a,
    {
        Self::List(Box::new(iter.into_iter().map(Ok)))
    }
}
