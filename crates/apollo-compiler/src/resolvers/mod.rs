use crate::executable;
use crate::response::JsonMap;
use futures::future::BoxFuture;
use futures::stream::BoxStream;
use serde_json_bytes::Value as JsonValue;

/// A concrete GraphQL object whose fields can be resolved during execution.
pub trait ObjectValue {
    /// Returns the name of the concrete object type
    ///
    /// That name expected to be that of an object type defined in the schema.
    /// This is called when the schema indicates an abstract (interface or union) type.
    fn type_name(&self) -> &str;

    /// Resolves a concrete field of this object
    ///
    /// `arguments` is the result of
    /// [`CoerceArgumentValues()`](https://spec.graphql.org/draft/#sec-Coercing-Field-Arguments`):
    /// when `resolve_field` is called its structure matches the argument definitions in the schema.
    ///
    /// The resolved value is expected to match the type of the corresponding field definition
    /// in the schema.
    ///
    /// This is _not_ called for [introspection](https://spec.graphql.org/draft/#sec-Introspection)
    /// meta-fields `__typename`, `__type`, or `__schema`: those are handled separately.
    fn resolve_field<'a>(
        &'a self,
        field: &'a executable::Field,
        arguments: &'a JsonMap,
    ) -> Result<ResolvedValue<'a>, ResolveError>;

    fn unknown_field_error(&self, field: &executable::Field) -> ResolveError {
        ResolveError::unknown_field(field, self.type_name())
    }
}

/// A concrete GraphQL object whose fields can be resolved asynchronously during execution.
pub trait AsyncObjectValue {
    /// Returns the name of the concrete object type
    ///
    /// That name expected to be that of an object type defined in the schema.
    /// This is called when the schema indicates an abstract (interface or union) type.
    fn type_name(&self) -> &str;

    /// Resolves a concrete field of this object
    ///
    /// `arguments` is the result of
    /// [`CoerceArgumentValues()`](https://spec.graphql.org/draft/#sec-Coercing-Field-Arguments`):
    /// when `resolve_field` is called its structure matches the argument definitions in the schema.
    ///
    /// The resolved value is expected to match the type of the corresponding field definition
    /// in the schema.
    ///
    /// This is _not_ called for [introspection](https://spec.graphql.org/draft/#sec-Introspection)
    /// meta-fields `__typename`, `__type`, or `__schema`: those are handled separately.
    fn resolve_field<'a>(
        &'a self,
        field: &'a executable::Field,
        arguments: &'a JsonMap,
    ) -> BoxFuture<'a, Result<ResolvedValue<'a>, ResolveError>>;

    fn unknown_field_error(&self, field: &executable::Field) -> ResolveError {
        ResolveError::unknown_field(field, self.type_name())
    }
}

pub struct ResolveError {
    pub message: String,
}

impl ResolveError {
    fn unknown_field(field: &executable::Field, type_name: &str) -> Self {
        Self {
            message: format!("unexpected field name: {} in type {type_name}", field.name),
        }
    }
}

/// The value of a resolved field
pub enum ResolvedValue<'a> {
    /// * JSON null represents GraphQL null
    /// * A GraphQL enum value is represented as a JSON string
    /// * GraphQL built-in scalars are coerced according to their respective *Result Coercion* spec
    /// * For custom scalars, any JSON value is passed through as-is (including array or object)
    Leaf(JsonValue),

    /// Expected where the GraphQL type is an object, interface, or union type
    Object(Box<dyn ObjectValue + 'a>),

    /// Expected for GraphQL list types
    List(Box<dyn Iterator<Item = Result<Self, ResolveError>> + 'a>),
}

/// The value of an asynchronously-resolved field
pub enum AsyncResolvedValue<'a> {
    /// * JSON null represents GraphQL null
    /// * A GraphQL enum value is represented as a JSON string
    /// * GraphQL built-in scalars are coerced according to their respective *Result Coercion* spec
    /// * For custom scalars, any JSON value is passed through as-is (including array or object)
    Leaf(JsonValue),

    /// Expected where the GraphQL type is an object, interface, or union type
    Object(Box<dyn AsyncObjectValue + 'a>),

    /// Expected for GraphQL list types
    List(BoxStream<'a, Result<Self, ResolveError>>),
}

impl<'a> ResolvedValue<'a> {
    /// Construct a null leaf resolved value
    pub fn null() -> Self {
        Self::Leaf(JsonValue::Null)
    }

    /// Construct a leaf resolved value from something that is convertible to JSON
    pub fn leaf(json: impl Into<JsonValue>) -> Self {
        Self::Leaf(json.into())
    }

    /// Construct an object resolved value
    pub fn object(object: impl ObjectValue + 'a) -> Self {
        Self::Object(Box::new(object))
    }

    /// Construct an object resolved value or null
    pub fn nullable_object(opt_object: Option<impl ObjectValue + 'a>) -> Self {
        match opt_object {
            Some(object) => Self::Object(Box::new(object)),
            None => Self::null(),
        }
    }

    /// Construct a list resolved value from an iterator
    ///
    /// If errors can happen during iteration,
    /// construct the [`ResolvedValue::List`] enum variant directly instead.
    pub fn list<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = Self>,
        I::IntoIter: 'a,
    {
        Self::List(Box::new(iter.into_iter().map(Ok)))
    }
}

impl<'a> AsyncResolvedValue<'a> {
    /// Construct a null leaf resolved value
    pub fn null() -> Self {
        Self::Leaf(JsonValue::Null)
    }

    /// Construct a leaf resolved value from something that is convertible to JSON
    pub fn leaf(json: impl Into<JsonValue>) -> Self {
        Self::Leaf(json.into())
    }

    /// Construct an object resolved value
    pub fn object(object: impl AsyncObjectValue + 'a) -> Self {
        Self::Object(Box::new(object))
    }

    /// Construct an object resolved value or null
    pub fn nullable_object(opt_object: Option<impl AsyncObjectValue + 'a>) -> Self {
        match opt_object {
            Some(object) => Self::Object(Box::new(object)),
            None => Self::null(),
        }
    }

    /// Construct a list resolved value from an iterator
    ///
    /// If errors can happen during iteration,
    /// construct the [`ResolvedValue::List`] enum variant directly instead.
    pub fn list<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = Self>,
        I::IntoIter: 'a + Send,
    {
        Self::List(Box::pin(futures::stream::iter(iter.into_iter().map(Ok))))
    }
}
