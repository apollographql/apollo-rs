//! GraphQL [requests](https://spec.graphql.org/draft/#request)
//!
//! This exists primarily to support [`introspection::partial_execute`].

use crate::executable::Operation;
#[cfg(doc)]
use crate::introspection;
use crate::parser::SourceMap;
use crate::parser::SourceSpan;
use crate::resolvers::input_coercion::InputCoercionError;
use crate::response::GraphQLError;
use crate::response::JsonMap;
use crate::validation::SuspectedValidationBug;
use crate::validation::Valid;
use crate::Schema;

/// Coerce the values of [variables](https://spec.graphql.org/draft/#sec-Language.Variables)
/// from a GraphQL request to the types expected by the operation.
///
/// This is [_CoerceVariableValues()_](https://spec.graphql.org/October2021/#CoerceVariableValues())
/// in the GraphQL specification.
///
/// Returns a [request error](https://spec.graphql.org/draft/#sec-Errors.Request-Errors)
/// if a value as an incompatible type, or if a required variable is not provided.
pub fn coerce_variable_values(
    schema: &Valid<Schema>,
    operation: &Operation,
    values: &JsonMap,
) -> Result<Valid<JsonMap>, RequestError> {
    Ok(crate::resolvers::input_coercion::coerce_variable_values(
        schema, operation, values,
    )?)
}

/// A [request error](https://spec.graphql.org/draft/#sec-Errors.Request-Errors) is an error
/// raised during an early phase of the [execution](https://spec.graphql.org/draft/#sec-Execution)
/// to indicate that the request as a whole is considered faulty.
///
/// A request error should cause the rest of execution to be aborted,
/// and result in a GraphQL response that does not have a `data` key.
/// This differs from a response with `"data": null` which can happen with
/// a [field error](https://spec.graphql.org/draft/#sec-Errors.Field-Errors)
/// on a non-null field whose ancestors fields are all also non-null.
/// In that case the `null` value
/// is [propagated](https://spec.graphql.org/draft/#sec-Handling-Field-Errors)
/// all the way to the entire response data.
#[derive(Debug, Clone)]
pub struct RequestError {
    pub(crate) message: String,
    pub(crate) location: Option<SourceSpan>,
    pub(crate) is_suspected_validation_bug: bool,
}

impl From<InputCoercionError> for RequestError {
    fn from(error: InputCoercionError) -> Self {
        match error {
            InputCoercionError::SuspectedValidationBug(SuspectedValidationBug {
                message,
                location,
            }) => Self {
                message,
                location,
                is_suspected_validation_bug: true,
            },
            InputCoercionError::ValueError { message, location } => Self {
                message,
                location,
                is_suspected_validation_bug: false,
            },
        }
    }
}

impl RequestError {
    pub fn message(&self) -> impl std::fmt::Display + '_ {
        &self.message
    }

    pub fn location(&self) -> Option<SourceSpan> {
        self.location
    }

    pub fn to_graphql_error(&self, sources: &SourceMap) -> GraphQLError {
        let mut error = GraphQLError::new(&self.message, self.location, sources);
        if self.is_suspected_validation_bug {
            error
                .extensions
                .insert("APOLLO_SUSPECTED_VALIDATION_BUG", true.into());
        }
        error
    }
}
