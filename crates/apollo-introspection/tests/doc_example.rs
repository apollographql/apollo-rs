use apollo_compiler::ExecutableDocument;
use apollo_compiler::Schema;
use apollo_introspection::get_operation;
use apollo_introspection::JsonMap;
use apollo_introspection::RequestErrorResponse;
use apollo_introspection::Response;
use apollo_introspection::SchemaIntrospectionQuery;
use apollo_introspection::VariableValues;

/// `schema` and `document` are presumed valid
pub fn execute_request(
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
    let response =
        execute_non_introspection(schema, &document, operation_name, &coerced_variable_values)?;
    let intropsection_response = introspection.execute_sync(schema, &coerced_variable_values)?;
    Ok(response.merge(intropsection_response))
}

fn execute_non_introspection(
    _schema: &Schema,
    _document: &ExecutableDocument,
    _operation_name: Option<&str>,
    _variable_values: &VariableValues,
) -> Result<Response, RequestErrorResponse> {
    unimplemented!()
}
