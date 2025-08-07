//! Partial execition of the
//! [schema introspection](https://spec.graphql.org/draft/#sec-Schema-Introspection)
//! portion of a query
//!
//! The main entry point is [`partial_execute`].

use crate::collections::HashMap;
use crate::executable::Operation;
#[cfg(doc)]
use crate::executable::OperationMap;
#[cfg(doc)]
use crate::request::coerce_variable_values;
use crate::request::RequestError;
use crate::resolvers::Execution;
use crate::resolvers::ObjectValue;
use crate::resolvers::ResolveError;
use crate::resolvers::ResolveInfo;
use crate::resolvers::ResolvedValue;
use crate::response::ExecutionResponse;
use crate::response::JsonMap;
use crate::schema::Implementers;
use crate::validation::Valid;
use crate::ExecutableDocument;
use crate::Name;
use crate::Schema;

mod max_depth;
pub(crate) mod resolvers;

/// Check that the nesting level of some list fields does not exceed a fixed depth limit.
///
/// Since [the schema-introspection schema][s] is recursive,
/// a malicious query could cause huge responses that grow exponentially to the nesting depth.
///
/// An error result is a [request error](https://spec.graphql.org/draft/#request-error):
/// execution must not run at all,
/// and the GraphQL response must not have a `data` key (which is different from `data: null`).
///
/// The exact criteria may change in future apollo-compiler versions.
///
/// [s]: https://spec.graphql.org/draft/#sec-Schema-Introspection.Schema-Introspection-Schema
pub fn check_max_depth(
    document: &Valid<ExecutableDocument>,
    operation: &Operation,
) -> Result<(), RequestError> {
    let initial_depth = 0;
    max_depth::check_selection_set(
        document,
        &mut HashMap::default(),
        initial_depth,
        &operation.selection_set,
    )
    .map(drop)
}

/// Excecutes the [schema introspection](https://spec.graphql.org/draft/#sec-Schema-Introspection)
/// portion of a query and returns a partial response.
///
/// * Consider calling [`check_max_depth`] before this function
/// * `implementers_map` is expected to be form
///   [`schema.implementers_map()`][Schema::implementers_map],
///   allowing it to be computed once and reused for many queries
/// * `operation` is expected to be from
///   [`document.operations.get(operation_name)?`][OperationMap::get]
/// * `operation` is expected to be a query,
///   check [`operation.operation_type.is_query()`][Operation::operation_type]
/// * `variable_values` is expected to be from [`coerce_variable_values`]
///
/// Concrete [root fields][Operation::root_fields] (those with an explicit definition in the schema)
/// are **_silently ignored_**.
///
/// Only introspection meta-fields are executed:
/// `__typename` (at the response root), `__type`, and `__schema`.
/// If the operation also contains concrete fields,
/// the caller can execute them separately and merge the two partial responses.
/// To categorize which kinds of root fields are present, consider using code like:
///
/// ```
/// # use apollo_compiler::executable as exe;
/// # fn categorize_fields(document: &exe::ExecutableDocument, operation: &exe::Operation) {
/// let mut has_schema_introspection_fields = false;
/// let mut has_root_typename_fields = false;
/// let mut has_concrete_fields = false;
/// for root_field in operation.root_fields(document) {
///     match root_field.name.as_str() {
///         "__type" | "__schema" => has_schema_introspection_fields = true,
///         "__typename" => has_root_typename_fields = true,
///         _ => has_concrete_fields = true,
///     }
/// }
/// # }
/// ```
pub fn partial_execute(
    schema: &Valid<Schema>,
    implementers_map: &HashMap<Name, Implementers>,
    document: &Valid<ExecutableDocument>,
    operation: &Operation,
    variable_values: &Valid<JsonMap>,
) -> Result<ExecutionResponse, RequestError> {
    struct InitialValue<'a> {
        type_name: &'a str,
    }

    impl ObjectValue for InitialValue<'_> {
        fn type_name(&self) -> &str {
            self.type_name
        }

        fn resolve_field<'a>(
            &'a self,
            _info: &'a ResolveInfo<'a>,
        ) -> Result<ResolvedValue<'a>, ResolveError> {
            // Introspection meta-fields are handled separately
            // so this is only called for concrete fields of the root query type
            Ok(ResolvedValue::SkipForPartialExecution)
        }
    }

    let initial_value = InitialValue {
        type_name: operation.object_type(),
    };
    Execution::new(schema, document)
        .implementers_map(implementers_map)
        .operation(operation)
        .coerced_variable_values(variable_values)
        .enable_schema_introspection(true)
        .execute_sync(&initial_value)
}
