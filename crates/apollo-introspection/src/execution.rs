#![allow(clippy::too_many_arguments)]

use crate::input_coercion::coerce_argument_values;
use crate::input_coercion::VariableValues;
use crate::resolver::ObjectValue;
use crate::response::field_error;
use crate::response::request_error;
use crate::response::Error;
use crate::response::LinkedPath;
use crate::response::LinkedPathElement;
use crate::response::PathElement;
use crate::response::RequestErrorResponse;
use crate::response::Response;
use crate::result_coercion::complete_value;
use crate::JsonMap;
use apollo_compiler::executable::Field;
use apollo_compiler::executable::Operation;
use apollo_compiler::executable::OperationType;
use apollo_compiler::executable::Selection;
use apollo_compiler::schema::ExtendedType;
use apollo_compiler::schema::FieldDefinition;
use apollo_compiler::schema::Name;
use apollo_compiler::schema::ObjectType;
use apollo_compiler::schema::Type;
use apollo_compiler::schema::Value;
use apollo_compiler::ExecutableDocument;
use apollo_compiler::Node;
use apollo_compiler::Schema;
use futures::future::join_all;
use indexmap::IndexMap;
use serde_json_bytes::Value as JsonValue;
use std::collections::HashSet;

/// <https://spec.graphql.org/October2021/#sec-Normal-and-Serial-Execution>
#[derive(Debug, Copy, Clone)]
pub(crate) enum ExecutionMode {
    /// Allowed to resolve fields in any order, including in parellel
    Normal,
    Sequential,
}

/// Return in `Err` when a field error occurred at some non-nullable place
///
/// <https://spec.graphql.org/October2021/#sec-Handling-Field-Errors>
pub(crate) struct PropagateNull;

/// Select one operation from a document, based on an optional requested operation name
///
/// <https://spec.graphql.org/October2021/#GetOperation()>
pub fn get_operation<'doc>(
    document: &'doc ExecutableDocument,
    operation_name: Option<&str>,
) -> Result<&'doc Node<Operation>, RequestErrorResponse> {
    document.get_operation(operation_name).map_err(|_| {
        if let Some(name) = operation_name {
            request_error(format!("no operation named '{name}'"))
        } else {
            request_error("multiple operations but no `operationName`")
        }
    })
}

/// The entry point of execution, after preparing with [`get_operation`]
/// and [`VariableValues::coerce`]
///
/// * <https://spec.graphql.org/October2021/#ExecuteQuery()>
/// * <https://spec.graphql.org/October2021/#ExecuteQuery()>
///
/// `schema` and `document` are presumed valid
pub(crate) async fn execute_query_or_mutation(
    schema: &Schema,
    document: &ExecutableDocument,
    variable_values: &VariableValues,
    initial_value: &ObjectValue<'_>,
    operation: &Operation,
) -> Result<Response, RequestErrorResponse> {
    let object_type_name = operation.object_type();
    let object_type_def = schema.get_object(object_type_name).ok_or_else(|| {
        request_error(format!(
            "Root operation type {object_type_name} is undefined or not an object type."
        ))
        .validation_should_have_caught_this()
    })?;
    let mut errors = Vec::new();
    let path = None; // root: empty path
    let mode = if operation.operation_type == OperationType::Mutation {
        ExecutionMode::Sequential
    } else {
        ExecutionMode::Normal
    };
    let data = execute_selection_set(
        schema,
        document,
        variable_values,
        &mut errors,
        path,
        mode,
        object_type_name,
        object_type_def,
        initial_value,
        &operation.selection_set.selections,
    )
    .await
    .ok();
    Ok(Response { data, errors })
}

/// <https://spec.graphql.org/October2021/#ExecuteSelectionSet()>
pub(crate) async fn execute_selection_set<'a>(
    schema: &Schema,
    document: &'a ExecutableDocument,
    variable_values: &VariableValues,
    errors: &mut Vec<Error>,
    path: LinkedPath<'_>,
    mode: ExecutionMode,
    object_type_name: &str,
    object_type: &ObjectType,
    object_value: &ObjectValue<'_>,
    selections: impl IntoIterator<Item = &'a Selection>,
) -> Result<JsonMap, PropagateNull> {
    let mut grouped_field_set = IndexMap::new();
    collect_fields(
        schema,
        document,
        variable_values,
        object_type_name,
        object_type,
        selections,
        &mut HashSet::new(),
        &mut grouped_field_set,
    );
    let futures = grouped_field_set
        .iter()
        .filter_map(|(&response_key, fields)| {
            // Indexing should not panic: `collect_fields` only creates a `Vec` to push to it
            let field_name = &fields[0].name;
            let Ok(field_def) = schema.type_field(object_type_name, field_name) else {
                // TODO: Return a validation_should_have_caught_this field error here?
                // The spec specifically has a “If fieldType is defined” condition,
                // but it being undefined would make the request invalid, right?
                return None;
            };
            Some(async move {
                let mut errors = Vec::new();
                let result = if field_name == "__typename" {
                    Ok(object_type_name.into())
                } else {
                    let field_path = LinkedPathElement {
                        element: PathElement::Field(response_key.clone()),
                        next: path,
                    };
                    execute_field(
                        schema,
                        document,
                        variable_values,
                        &mut errors,
                        Some(&field_path),
                        mode,
                        object_value,
                        field_def,
                        fields,
                    )
                    .await
                };
                (response_key, result, errors)
            })
        });
    let mut response_map = JsonMap::with_capacity(grouped_field_set.len());
    match mode {
        ExecutionMode::Normal => {
            // `join_all` executes fields concurrently but preserves their ordering
            let outputs = join_all(futures).await;
            for (response_key, result, mut field_errors) in outputs {
                errors.append(&mut field_errors);
                response_map.insert(response_key.as_str(), result?);
            }
        }
        ExecutionMode::Sequential => {
            // Only start executing one field after the previous on is finished
            for future in futures {
                let (response_key, result, mut field_errors) = future.await;
                errors.append(&mut field_errors);
                response_map.insert(response_key.as_str(), result?);
            }
        }
    }
    Ok(response_map)
}

/// <https://spec.graphql.org/October2021/#CollectFields()>
fn collect_fields<'a>(
    schema: &Schema,
    document: &'a ExecutableDocument,
    variable_values: &VariableValues,
    object_type_name: &str,
    object_type: &ObjectType,
    selections: impl IntoIterator<Item = &'a Selection>,
    visited_fragments: &mut HashSet<&'a Name>,
    grouped_fields: &mut IndexMap<&'a Name, Vec<&'a Field>>,
) {
    for selection in selections {
        if eval_if_arg(selection, "skip", variable_values).unwrap_or(false)
            || !eval_if_arg(selection, "include", variable_values).unwrap_or(true)
        {
            continue;
        }
        match selection {
            Selection::Field(field) => grouped_fields
                .entry(field.response_key())
                .or_default()
                .push(field.as_ref()),
            Selection::FragmentSpread(spread) => {
                let new = visited_fragments.insert(&spread.fragment_name);
                if !new {
                    continue;
                }
                let Some(fragment) = document.fragments.get(&spread.fragment_name) else {
                    continue;
                };
                if !does_fragment_type_apply(
                    schema,
                    object_type_name,
                    object_type,
                    fragment.type_condition(),
                ) {
                    continue;
                }
                collect_fields(
                    schema,
                    document,
                    variable_values,
                    object_type_name,
                    object_type,
                    &fragment.selection_set.selections,
                    visited_fragments,
                    grouped_fields,
                )
            }
            Selection::InlineFragment(inline) => {
                if let Some(condition) = &inline.type_condition {
                    if !does_fragment_type_apply(schema, object_type_name, object_type, condition) {
                        continue;
                    }
                }
                collect_fields(
                    schema,
                    document,
                    variable_values,
                    object_type_name,
                    object_type,
                    &inline.selection_set.selections,
                    visited_fragments,
                    grouped_fields,
                )
            }
        }
    }
}

/// <https://spec.graphql.org/October2021/#DoesFragmentTypeApply()>
fn does_fragment_type_apply(
    schema: &Schema,
    object_type_name: &str,
    object_type: &ObjectType,
    fragment_type: &Name,
) -> bool {
    match schema.types.get(fragment_type) {
        Some(ExtendedType::Object(_)) => fragment_type == object_type_name,
        Some(ExtendedType::Interface(_)) => {
            object_type.implements_interfaces.contains(fragment_type)
        }
        Some(ExtendedType::Union(def)) => def.members.contains(object_type_name),
        // Undefined or not an output type: validation should have caught this
        _ => false,
    }
}

fn eval_if_arg(
    selection: &Selection,
    directive_name: &str,
    variable_values: &VariableValues,
) -> Option<bool> {
    match selection
        .directives()
        .get(directive_name)?
        .argument_by_name("if")?
        .as_ref()
    {
        Value::Boolean(value) => Some(*value),
        Value::Variable(var) => variable_values.get(var.as_str())?.as_bool(),
        _ => None,
    }
}

/// <https://spec.graphql.org/October2021/#ExecuteField()>
async fn execute_field(
    schema: &Schema,
    document: &ExecutableDocument,
    variable_values: &VariableValues,
    errors: &mut Vec<Error>,
    path: LinkedPath<'_>,
    mode: ExecutionMode,
    object_value: &ObjectValue<'_>,
    field_def: &FieldDefinition,
    fields: &[&Field],
) -> Result<JsonValue, PropagateNull> {
    let field = fields[0];
    let argument_values =
        match coerce_argument_values(schema, variable_values, errors, path, field_def, field) {
            Ok(argument_values) => argument_values,
            Err(PropagateNull) => return try_nullify(&field_def.ty, Err(PropagateNull)),
        };
    let resolved_result = object_value
        .resolve_field(&field.name, &argument_values)
        .await;
    let completed_result = match resolved_result {
        Ok(resolved) => {
            complete_value(
                schema,
                document,
                variable_values,
                errors,
                path,
                mode,
                field.ty(),
                resolved,
                fields,
            )
            .await
        }
        Err(message) => {
            errors.push(field_error(
                format!("resolver error: {message}"),
                path,
                field.name.location(),
            ));
            Err(PropagateNull)
        }
    };
    try_nullify(&field_def.ty, completed_result)
}

/// Try to insert a propagated null if possible, or keep propagating it.
///
/// <https://spec.graphql.org/October2021/#sec-Handling-Field-Errors>
pub(crate) fn try_nullify(
    ty: &Type,
    result: Result<JsonValue, PropagateNull>,
) -> Result<JsonValue, PropagateNull> {
    match result {
        Ok(json) => Ok(json),
        Err(PropagateNull) => {
            if ty.is_non_null() {
                Err(PropagateNull)
            } else {
                Ok(JsonValue::Null)
            }
        }
    }
}
