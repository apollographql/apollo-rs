use crate::collections::HashMap;
use crate::executable;
use crate::executable::Operation;
use crate::execution::engine::execute_selection_set;
use crate::execution::engine::ExecutionContext;
use crate::execution::engine::ExecutionMode;
use crate::execution::engine::MaybeLazy;
use crate::execution::engine::PropagateNull;
#[cfg(doc)]
use crate::introspection;
use crate::request::coerce_variable_values;
use crate::request::RequestError;
use crate::response::ExecutionResponse;
use crate::response::JsonMap;
use crate::response::JsonValue;
use crate::schema;
use crate::schema::Implementers;
use crate::validation::Valid;
use crate::ExecutableDocument;
use crate::Name;
use crate::Schema;
use futures::future::BoxFuture;
use futures::stream::BoxStream;
use futures::FutureExt as _;
use std::cell::OnceCell;

pub struct Execution<'a> {
    schema: &'a Valid<Schema>,
    document: &'a Valid<ExecutableDocument>,
    operation: Option<&'a Operation>,
    implementers_map: Option<&'a HashMap<Name, Implementers>>,
    variable_values: Option<VariableValues<'a>>,
    enable_schema_introspection: Option<bool>,
}

/// Default to disabled:
/// https://www.apollographql.com/blog/why-you-should-disable-graphql-introspection-in-production/
const DEFAULT_ENABLE_SCHEMA_INTROSPECTION: bool = false;

enum VariableValues<'a> {
    Raw(&'a JsonMap),
    Coerced(&'a Valid<JsonMap>),
}

#[derive(Clone, Copy)]
pub(crate) enum MaybeAsync<A, S> {
    Async(A),
    Sync(S),
}

pub(crate) type MaybeAsyncObject<'a> = MaybeAsync<&'a dyn AsyncObjectValue, &'a dyn ObjectValue>;

pub(crate) type MaybeAsyncResolved<'a> = MaybeAsync<AsyncResolvedValue<'a>, ResolvedValue<'a>>;

/// Information passed to resolvers
pub struct ResolveInfo<'a> {
    pub(crate) schema: &'a Valid<Schema>,
    pub(crate) implementers_map: MaybeLazy<'a, HashMap<Name, Implementers>>,
    pub(crate) document: &'a Valid<ExecutableDocument>,
    pub(crate) fields: &'a [&'a executable::Field],
    pub(crate) arguments: &'a JsonMap,
}

pub struct ResolveError {
    pub message: String,
}

/// A concrete GraphQL object whose fields can be resolved during execution.
pub trait ObjectValue {
    /// Returns the name of the concrete object type
    ///
    /// That name expected to be that of an object type defined in the schema.
    /// This is called when the schema indicates an abstract (interface or union) type.
    fn type_name(&self) -> &str;

    /// Resolves a concrete field of this object
    ///
    /// The resolved value is expected to match the type of the corresponding field definition
    /// in the schema.
    ///
    /// This is _not_ called for [introspection](https://spec.graphql.org/draft/#sec-Introspection)
    /// meta-fields `__typename`, `__type`, or `__schema`: those are handled separately.
    fn resolve_field<'a>(
        &'a self,
        info: &ResolveInfo<'a>,
    ) -> Result<ResolvedValue<'a>, ResolveError>;

    fn unknown_field_error(&self, info: &ResolveInfo<'_>) -> ResolveError {
        ResolveError::unknown_field(info.field_name(), self.type_name())
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
    /// The resolved value is expected to match the type of the corresponding field definition
    /// in the schema.
    ///
    /// This is _not_ called for [introspection](https://spec.graphql.org/draft/#sec-Introspection)
    /// meta-fields `__typename`, `__type`, or `__schema`: those are handled separately.
    fn resolve_field<'a>(
        &'a self,
        info: &ResolveInfo<'a>,
    ) -> BoxFuture<'a, Result<AsyncResolvedValue<'a>, ResolveError>>;

    fn unknown_field_error(&self, info: &ResolveInfo<'_>) -> ResolveError {
        ResolveError::unknown_field(info.field_name(), self.type_name())
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

    /// Skip this field as if the selection had `@skip(if: true)`:
    /// do not insert null nor emit an error.
    ///
    /// This causes the eventual response data to be incomplete.
    /// This can be useful to have some fields executed with per-field resolvers by this API
    /// and other fields with some other execution model such as Apollo Federation,
    /// with the two response `data` maps merged before sending the response.
    ///
    /// This is used by [`introspection::partial_execute`].
    SkipForPartialExcecution,
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

    /// Skip this field as if the selection had `@skip(if: true)`:
    /// do not insert null nor emit an error.
    ///
    /// This causes the eventual response data to be incomplete.
    /// This can be useful to have some fields executed with per-field resolvers by this API
    /// and other fields with some other execution model such as Apollo Federation,
    /// with the two response `data` maps merged before sending the response.
    ///
    /// This is used by [`introspection::partial_execute`].
    SkipForPartialExcecution,
}

impl<'a> Execution<'a> {
    pub fn new(schema: &'a Valid<Schema>, document: &'a Valid<ExecutableDocument>) -> Self {
        Self {
            schema,
            document,
            operation: None,
            implementers_map: None,
            variable_values: None,
            enable_schema_introspection: None,
        }
    }

    pub fn operation(mut self, operation: &'a Operation) -> Self {
        assert!(
            self.operation.is_none(),
            "operation to execute already provided"
        );
        self.operation = Some(operation);
        self
    }

    pub fn operation_name(mut self, operation_name: Option<&str>) -> Result<Self, RequestError> {
        assert!(
            self.operation.is_none(),
            "operation to execute already provided"
        );
        self.operation = Some(self.document.operations.get(operation_name)?);
        Ok(self)
    }

    pub fn implementers_map(mut self, implementers_map: &'a HashMap<Name, Implementers>) -> Self {
        assert!(
            self.implementers_map.is_none(),
            "implementers map already provided"
        );
        self.implementers_map = Some(implementers_map);
        self
    }

    pub fn coerced_variable_values(mut self, variable_values: &'a Valid<JsonMap>) -> Self {
        assert!(
            self.variable_values.is_none(),
            "variable values already provided"
        );
        self.variable_values = Some(VariableValues::Coerced(variable_values));
        self
    }

    pub fn raw_variable_values(mut self, variable_values: &'a JsonMap) -> Self {
        assert!(
            self.variable_values.is_none(),
            "variable values already provided"
        );
        self.variable_values = Some(VariableValues::Raw(variable_values));
        self
    }

    pub fn enable_schema_introspection(mut self, enable_schema_introspection: bool) -> Self {
        assert!(
            self.enable_schema_introspection.is_none(),
            "schema introspection already configured"
        );
        self.enable_schema_introspection = Some(enable_schema_introspection);
        self
    }

    pub fn execute_sync(
        &self,
        initial_value: &dyn ObjectValue,
    ) -> Result<ExecutionResponse, RequestError> {
        let future = self.execute_common(MaybeAsync::Sync(initial_value));

        // An `async fn` returns a future whose `poll` method returns:
        //
        // * `Poll::Ready(R)` when the function returns
        // * `Poll::Pending` when it `.await`s an inner future that returns `Poll::Pending`
        //
        // When we use `MaybeAsync::Sync`, there are no manually-written implementations
        // of the `Future` trait involved at all, only `async fn`s that call each other.
        // Therefore we expect `Poll::Pending` to never be generated.
        // Instead futures should resolve immediately and `now_or_never` should never return `None`.
        future
            .now_or_never()
            .expect("expected async fn with sync resolvers to never be pending")
    }

    pub async fn execute_async(
        &self,
        initial_value: &dyn AsyncObjectValue,
    ) -> Result<ExecutionResponse, RequestError> {
        self.execute_common(MaybeAsync::Async(initial_value)).await
    }

    async fn execute_common(
        &self,
        initial_value: MaybeAsyncObject<'_>,
    ) -> Result<ExecutionResponse, RequestError> {
        let operation = if let Some(op) = self.operation {
            op
        } else {
            self.document.operations.get(None)?
        };

        let object_type_name = operation.object_type();
        let Some(root_operation_object_type_def) = self.schema.get_object(object_type_name) else {
            return Err(RequestError {
                message: "Undefined root operation type".to_owned(),
                location: object_type_name.location(),
                is_suspected_validation_bug: true,
            });
        };

        let map;
        let variable_values = match self.variable_values {
            None => {
                map = Valid::assume_valid(JsonMap::new());
                &map
            }
            Some(VariableValues::Raw(v)) => {
                map = coerce_variable_values(self.schema, operation, v)?;
                &map
            }
            Some(VariableValues::Coerced(v)) => v,
        };
        let cell;
        let implementers_map = match self.implementers_map {
            None => {
                cell = OnceCell::new();
                MaybeLazy::Lazy(&cell)
            }
            Some(map) => MaybeLazy::Eager(map),
        };
        let enable_schema_introspection = self
            .enable_schema_introspection
            .unwrap_or(DEFAULT_ENABLE_SCHEMA_INTROSPECTION);
        let mut errors = Vec::new();
        let mut context = ExecutionContext {
            schema: self.schema,
            document: self.document,
            variable_values,
            errors: &mut errors,
            implementers_map,
            enable_schema_introspection,
        };
        let mode = match operation.operation_type {
            executable::OperationType::Query | executable::OperationType::Subscription => {
                ExecutionMode::Normal
            }
            executable::OperationType::Mutation => ExecutionMode::Sequential,
        };
        let result = execute_selection_set(
            &mut context,
            None,
            mode,
            root_operation_object_type_def,
            initial_value,
            &operation.selection_set.selections,
        )
        .await;
        let data = result
            // If `Result::ok` converts an error to `None` that’s a field error on a non-null,
            // field propagated all the way to the root,
            // so that the JSON response should contain `"data": null`.
            //
            // No-op to witness the error type:
            .inspect_err(|_: &PropagateNull| {})
            .ok();
        Ok(ExecutionResponse { data, errors })
    }
}

impl<'a> ResolveInfo<'a> {
    // https://github.com/graphql/graphql-js/blob/v16.11.0/src/type/definition.ts#L980-L991

    pub fn schema(&self) -> &'a Valid<Schema> {
        self.schema
    }

    pub fn implementers_map(&self) -> &'a HashMap<Name, Implementers> {
        match self.implementers_map {
            MaybeLazy::Eager(map) => map,
            MaybeLazy::Lazy(cell) => cell.get_or_init(|| self.schema.implementers_map()),
        }
    }

    pub fn document(&self) -> &'a Valid<ExecutableDocument> {
        self.document
    }

    pub fn field_name(&self) -> &'a str {
        &self.fields[0].name
    }

    pub fn field_defintion(&self) -> &'a schema::FieldDefinition {
        &self.fields[0].definition
    }

    pub fn field_selections(&self) -> &'a [&'a executable::Field] {
        self.fields
    }

    /// The arguments passed to this field, after
    /// [`CoerceArgumentValues()`](https://spec.graphql.org/draft/#sec-Coercing-Field-Arguments`):
    /// this matches the argument definitions in the schema.
    pub fn arguments(&self) -> &'a JsonMap {
        self.arguments
    }
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

impl MaybeAsync<Box<dyn AsyncObjectValue + '_>, Box<dyn ObjectValue + '_>> {
    pub(crate) fn type_name(&self) -> &str {
        match self {
            MaybeAsync::Async(obj) => obj.type_name(),
            MaybeAsync::Sync(obj) => obj.type_name(),
        }
    }
}

impl ResolveError {
    fn unknown_field(field_name: &str, type_name: &str) -> Self {
        Self {
            message: format!("unexpected field name: {field_name} in type {type_name}"),
        }
    }
}
