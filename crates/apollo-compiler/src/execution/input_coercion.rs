use crate::ast::Type;
use crate::ast::Value;
use crate::executable::Operation;
use crate::execution::JsonMap;
use crate::execution::JsonValue;
use crate::execution::RequestError;
use crate::schema::ExtendedType;
use crate::validation::Valid;
use crate::ExecutableDocument;
use crate::Schema;

macro_rules! request_error {
    ($($arg: tt)+) => {
        return Err(RequestError::new(format_args!($($arg)+)))
    };
}

macro_rules! validation_bug {
    ($($arg: tt)+) => {
        return Err(RequestError::new(format_args!($($arg)+)).validation_bug())
    };
}

/// Coerce the values of variables from a GraphQL request to the types expected by the operation.
///
/// If type coercion fails, a request error is returned and the request must not be executed.
///
/// This is [CoerceVariableValues()](https://spec.graphql.org/October2021/#CoerceVariableValues())
/// in the GraphQL specification.
pub fn coerce_variable_values(
    schema: &Valid<Schema>,
    document: &Valid<ExecutableDocument>,
    operation_name: Option<&str>,
    values: &JsonMap,
) -> Result<Valid<JsonMap>, RequestError> {
    let operation = document.get_operation(operation_name)?;
    let mut coerced_values = JsonMap::new();
    for variable_def in &operation.variables {
        let name = variable_def.name.as_str();
        if let Some((key, value)) = values.get_key_value(name) {
            let value = coerce_variable_value(
                schema,
                document,
                "variable",
                "",
                "",
                name,
                &variable_def.ty,
                value,
            )?;
            coerced_values.insert(key.clone(), value);
        } else if let Some(default) = &variable_def.default_value {
            let value =
                graphql_value_to_json("variable", "", "", name, default).map_err(|mut err| {
                    err.0
                        .locations
                        .extend(default.line_column(&document.sources));
                    err
                })?;
            coerced_values.insert(name, value);
        } else if variable_def.ty.is_non_null() {
            request_error!("missing value for non-null variable '{name}'")
        } else {
            // Nullable variable with no provided value nor explicit default.
            // Spec says nothing for this case, but for the similar case in input objects:
            //
            // > there is a semantic difference between the explicitly provided value null
            // > versus having not provided a value
        }
    }
    Ok(Valid(coerced_values))
}

#[allow(clippy::too_many_arguments)] // yes it’s not a nice API but it’s internal
fn coerce_variable_value(
    schema: &Valid<Schema>,
    document: &Valid<ExecutableDocument>,
    kind: &str,
    parent: &str,
    sep: &str,
    name: &str,
    ty: &Type,
    value: &JsonValue,
) -> Result<JsonValue, RequestError> {
    if value.is_null() {
        if ty.is_non_null() {
            request_error!("null value for non-null {kind} {parent}{sep}{name}")
        } else {
            return Ok(JsonValue::Null);
        }
    }
    let ty_name = match ty {
        Type::List(inner) | Type::NonNullList(inner) => {
            // https://spec.graphql.org/October2021/#sec-List.Input-Coercion
            return value
                .as_array()
                .map(Vec::as_slice)
                // If not an array, treat the value as an array of size one:
                .unwrap_or(std::slice::from_ref(value))
                .iter()
                .map(|item| {
                    coerce_variable_value(schema, document, kind, parent, sep, name, inner, item)
                })
                .collect();
        }
        Type::Named(ty_name) | Type::NonNullNamed(ty_name) => ty_name,
    };
    let Some(ty_def) = schema.types.get(ty_name) else {
        validation_bug!("Undefined type {ty_name} for {kind} {parent}{sep}{name}")
    };
    match ty_def {
        ExtendedType::Object(_) | ExtendedType::Interface(_) | ExtendedType::Union(_) => {
            validation_bug!("Non-input type {ty_name} for {kind} {parent}{sep}{name}.")
        }
        ExtendedType::Scalar(_) => match ty_name.as_str() {
            "Int" => {
                // https://spec.graphql.org/October2021/#sec-Int.Input-Coercion
                if value
                    .as_i64()
                    .is_some_and(|value| i32::try_from(value).is_ok())
                {
                    return Ok(value.clone());
                }
            }
            "Float" => {
                // https://spec.graphql.org/October2021/#sec-Float.Input-Coercion
                if value.is_f64() {
                    return Ok(value.clone());
                }
            }
            "String" => {
                // https://spec.graphql.org/October2021/#sec-String.Input-Coercion
                if value.is_string() {
                    return Ok(value.clone());
                }
            }
            "Boolean" => {
                // https://spec.graphql.org/October2021/#sec-Boolean.Input-Coercion
                if value.is_boolean() {
                    return Ok(value.clone());
                }
            }
            "ID" => {
                // https://spec.graphql.org/October2021/#sec-ID.Input-Coercion
                if value.is_string() || value.is_i64() {
                    return Ok(value.clone());
                }
            }
            _ => {
                // Custom scalar
                // TODO: have a hook for coercion of custom scalars?
                return Ok(value.clone());
            }
        },
        ExtendedType::Enum(ty_def) => {
            // https://spec.graphql.org/October2021/#sec-Enums.Input-Coercion
            if let Some(str) = value.as_str() {
                if ty_def.values.keys().any(|value_name| value_name == str) {
                    return Ok(value.clone());
                }
            }
        }
        ExtendedType::InputObject(ty_def) => {
            // https://spec.graphql.org/October2021/#sec-Input-Objects.Input-Coercion
            if let Some(object) = value.as_object() {
                if let Some(key) = object
                    .keys()
                    .find(|key| !ty_def.fields.contains_key(key.as_str()))
                {
                    request_error!(
                        "Input object has key {} not in type {ty_name}",
                        key.as_str()
                    )
                }
                let mut object = object.clone();
                for (field_name, field_def) in &ty_def.fields {
                    if let Some(field_value) = object.get_mut(field_name.as_str()) {
                        *field_value = coerce_variable_value(
                            schema,
                            document,
                            "input field",
                            ty_name,
                            ".",
                            field_name,
                            &field_def.ty,
                            field_value,
                        )?
                    } else if let Some(default) = &field_def.default_value {
                        let default =
                            graphql_value_to_json("input field", ty_name, ".", field_name, default)
                                .map_err(|mut err| {
                                    err.0
                                        .locations
                                        .extend(default.line_column(&document.sources));
                                    err
                                })?;
                        object.insert(field_name.as_str(), default);
                    } else if field_def.ty.is_non_null() {
                        request_error!(
                            "Missing value for non-null input object field {ty_name}.{field_name}"
                        )
                    } else {
                        // Field not required
                    }
                }
                return Ok(object.into());
            }
        }
    }
    request_error!("Could not coerce {kind} {parent}{sep}{name}: {value} to type {ty_name}")
}

fn graphql_value_to_json(
    kind: &str,
    parent: &str,
    sep: &str,
    name: &str,
    value: &Value,
) -> Result<JsonValue, RequestError> {
    match value {
        Value::Null => Ok(JsonValue::Null),
        Value::Variable(_) => {
            // TODO: separate `ContValue` enum without this variant?
            validation_bug!("Variable in default value of {kind} {parent}{sep}{name}.")
        }
        Value::Enum(value) => Ok(value.as_str().into()),
        Value::String(value) => Ok(value.as_str().into()),
        Value::Boolean(value) => Ok((*value).into()),
        // Rely on `serde_json::Number`’s own parser to use whatever precision it supports
        Value::Int(value) => Ok(JsonValue::Number(value.as_str().parse().or_else(|_| {
            request_error!("Int value overflow in {kind} {parent}{sep}{name}")
        })?)),
        Value::Float(value) => Ok(JsonValue::Number(value.as_str().parse().or_else(|_| {
            request_error!("Float value overflow in {kind} {parent}{sep}{name}")
        })?)),
        Value::List(value) => value
            .iter()
            .map(|value| graphql_value_to_json(kind, parent, sep, name, value))
            .collect(),
        Value::Object(value) => value
            .iter()
            .map(|(key, value)| {
                Ok((
                    key.as_str(),
                    graphql_value_to_json(kind, parent, sep, name, value)?,
                ))
            })
            .collect(),
    }
}
