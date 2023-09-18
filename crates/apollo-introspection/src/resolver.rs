use crate::JsonMap;
use futures::future::BoxFuture;
use futures::stream;
use futures::stream::BoxStream;
use futures::Stream;
use serde_json_bytes::Value as JsonValue;

/// A GraphQL object whose fields can be resolved during execution
pub(crate) type ObjectValue<'a> = dyn Resolver + Send + 'a;

/// Abstraction for implementing field resolvers. Used through [`ObjectValue`].
///
/// Use the [`impl_resolver!`][crate::impl_resolver] macro to implement this trait
/// with reduced boilerplate
pub(crate) trait Resolver: Sync {
    /// Returns the name of the concrete object type this resolver represents
    ///
    /// That name expected to be that of an object type defined in the schema.
    /// This is called when the schema indicates an abstract (interface or union) type.
    fn type_name(&self) -> &'static str;

    /// Resolves a field of this object with the given arguments
    ///
    /// The resolved is expected to match the type of the corresponding field definition
    /// in the schema.
    fn resolve_field<'a>(
        &'a self,
        field_name: &'a str,
        arguments: &'a JsonMap,
    ) -> BoxFuture<'a, Result<ResolvedValue<'_>, String>>;
}

/// Implements the [`Resolver`] trait with reduced boilerplate
///
/// Define:
///
/// * The implementing Rust type
/// * The __typename string
/// * One async pseudo-method per field. Types are omitted in the signature for brevity.
///   - Takes two optional arguments: `&self` (which must be spelled something else because macros)
///     and `args: `[`&JsonMap`][crate::JsonMap] for the field arguments.
///     Field arguments are coerced according to their definition in the schema.
///   - Returns `Result<ResolvedValue, String>`, `Err` it turned into a field error
macro_rules! impl_resolver {
    (
        for $ty: ty:
        __typename = $type_name: expr;
        $(
            async fn $field_name: ident(
                $( &$self_: ident $(, $( $args: ident $(,)? )? )? )?
            ) $block: block
        )*

    ) => {
        impl $crate::resolver::Resolver for $ty {
            fn type_name(&self) -> &'static str {
                $type_name
            }

            fn resolve_field<'a>(
                &'a self,
                field_name: &'a str,
                arguments: &'a $crate::JsonMap,
            ) -> futures::future::BoxFuture<'a, Result<$crate::resolver::ResolvedValue<'_>, String>> {
                Box::pin(async move {
                    let _allow_unused = arguments;
                    match field_name {
                        $(
                            stringify!($field_name) => {
                                $(
                                    let $self_ = self;
                                    $($(
                                        let $args = arguments;
                                    )?)?
                                )?
                                return $block
                            },
                        )*
                        _ => Err(format!("unexpected field name: {field_name}")),
                    }
                })
            }
        }
    };
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

    /// Expected for GrapQL list types
    List(BoxStream<'a, ResolvedValue<'a>>),
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
    pub(crate) fn object(resolver: impl Resolver + Send + 'a) -> Self {
        Self::Object(Box::new(resolver))
    }

    /// Construct an object resolved value or null, from an optional resolver
    pub(crate) fn opt_object(opt_resolver: Option<impl Resolver + Send + 'a>) -> Self {
        match opt_resolver {
            Some(resolver) => Self::Object(Box::new(resolver)),
            None => Self::null(),
        }
    }

    /// Construct a list resolved value from an asynchronous stream
    pub(crate) fn list_stream<S>(stream: S) -> Self
    where
        S: Stream<Item = Self> + Send + 'a,
    {
        Self::List(Box::pin(stream))
    }

    /// Construct a list resolved value from an iterator
    pub(crate) fn list<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = Self>,
        I::IntoIter: Send + 'a,
    {
        Self::list_stream(stream::iter(iter))
    }
}

#[cfg(test)]
mod tests {
    use crate::execution::execute_query_or_mutation;
    use crate::execution::get_operation;
    use crate::resolver::ResolvedValue;
    use crate::JsonMap;
    use crate::RequestErrorResponse;
    use crate::Response;
    use crate::SchemaIntrospectionQuery;
    use crate::VariableValues;
    use apollo_compiler::executable::OperationType;
    use apollo_compiler::ExecutableDocument;
    use apollo_compiler::Schema;

    struct QueryResolver {
        world: String,
    }

    impl_resolver! {
        for &'_ QueryResolver:

        __typename = "Query";

        async fn null() {
            Ok(ResolvedValue::null())
        }

        async fn hello(&self_) {
            Ok(ResolvedValue::list([
                ResolvedValue::leaf(format!("Hello {}!", self_.world)),
                ResolvedValue::leaf(format!("Hello {}!", self_.world)),
            ]))
        }

        async fn echo(&_self, args) {
            Ok(ResolvedValue::leaf(args["value"].clone()))
        }

        async fn myself_again(&self_) {
            Ok(ResolvedValue::object(*self_))
        }
    }

    /// <https://spec.graphql.org/October2021/#sec-Executing-Requests>
    ///
    /// `schema` and `document` are presumed valid
    #[allow(unused)]
    async fn execute_request(
        schema: &Schema,
        mut document: ExecutableDocument,
        operation_name: Option<&str>,
        variable_values: &JsonMap,
    ) -> Result<Response, RequestErrorResponse> {
        let introspection = SchemaIntrospectionQuery::split_from(&mut document, operation_name)?;
        let operation = get_operation(&document, operation_name)?
            .definition()
            .clone();
        let coerced_variable_values = VariableValues::coerce(schema, &operation, variable_values)?;
        let initial_value = match operation.operation_type {
            OperationType::Query => QueryResolver {
                world: "World".into(),
            },
            _ => unimplemented!(),
        };
        let response = execute_query_or_mutation(
            schema,
            &document,
            &coerced_variable_values,
            &&initial_value,
            &operation,
        )
        .await?;
        let intropsection_response = introspection
            .execute(schema, &coerced_variable_values)
            .await?;
        Ok(response.merge(intropsection_response))
    }
}
