use crate::execution::PropagateNull;
use crate::response::field_error;
use crate::response::request_error;
use crate::response::to_locations;
use crate::response::Error;
use crate::response::LinkedPath;
use crate::response::RequestErrorResponse;
use crate::JsonMap;
use apollo_compiler::executable::Field;
use apollo_compiler::executable::Operation;
use apollo_compiler::schema::ExtendedType;
use apollo_compiler::schema::FieldDefinition;
use apollo_compiler::schema::Type;
use apollo_compiler::schema::Value;
use apollo_compiler::Node;
use apollo_compiler::Schema;
use serde_json_bytes::Value as JsonValue;
use std::collections::HashMap;

/// Values of variables from a given GraphQL request, after coercion to types expected by the operation.
pub struct VariableValues(JsonMap);

impl std::ops::Deref for VariableValues {
    type Target = JsonMap;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl VariableValues {
    /// <https://spec.graphql.org/October2021/#sec-Coercing-Variable-Values>
    ///
    /// `schema` and `document` are presumed valid
    pub fn coerce(
        schema: &Schema,
        operation: &Operation,
        values: &JsonMap,
    ) -> Result<Self, RequestErrorResponse> {
        coerce_variable_values(schema, operation, values)
    }
}

macro_rules! request_error {
    ($($arg: tt)+) => {
        return Err(request_error(format!($($arg)+)))
    };
}

macro_rules! validation_should_have_caught_this {
    ($($arg: tt)+) => {
        return Err(request_error(format!($($arg)+)).validation_should_have_caught_this())
    };
}

/// <https://spec.graphql.org/October2021/#CoerceVariableValues()>
fn coerce_variable_values(
    schema: &Schema,
    operation: &Operation,
    values: &JsonMap,
) -> Result<VariableValues, RequestErrorResponse> {
    let mut coerced_values = JsonMap::new();
    for variable_def in &operation.variables {
        let name = variable_def.name.as_str();
        if let Some((key, value)) = values.get_key_value(name) {
            let value =
                coerce_variable_value(schema, "variable", "", "", name, &variable_def.ty, value)?;
            coerced_values.insert(key.clone(), value);
        } else if let Some(default) = &variable_def.default_value {
            let value =
                graphql_value_to_json("variable", "", "", name, default).map_err(|mut err| {
                    err.errors[0].locations = to_locations(default.location());
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
    Ok(VariableValues(coerced_values))
}

fn coerce_variable_value(
    schema: &Schema,
    kind: &str,
    parent: &str,
    sep: &str,
    name: &str,
    ty: &Type,
    value: &JsonValue,
) -> Result<JsonValue, RequestErrorResponse> {
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
                .map(|item| coerce_variable_value(schema, kind, parent, sep, name, inner, item))
                .collect();
        }
        Type::Named(ty_name) | Type::NonNullNamed(ty_name) => ty_name,
    };
    let Some(ty_def) = schema.types.get(ty_name) else {
        validation_should_have_caught_this!(
            "Undefined type {ty_name} for {kind} {parent}{sep}{name}"
        )
    };
    match ty_def {
        ExtendedType::Object(_) | ExtendedType::Interface(_) | ExtendedType::Union(_) => {
            validation_should_have_caught_this!(
                "Non-input type {ty_name} for {kind} {parent}{sep}{name}."
            )
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
                                    err.errors[0].locations = to_locations(default.location());
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
) -> Result<JsonValue, RequestErrorResponse> {
    match value {
        Value::Null => Ok(JsonValue::Null),
        Value::Variable(_) => {
            // TODO: separate `ContValue` enum without this variant?
            validation_should_have_caught_this!(
                "Variable in default value of {kind} {parent}{sep}{name}."
            )
        }
        Value::Enum(value) => Ok(value.as_str().into()),
        Value::String(value) => Ok(value.as_str().into()),
        Value::Boolean(value) => Ok((*value).into()),
        // Rely on `serde_json::Number`’s own parser to use whateven precision it supports
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

/// <https://spec.graphql.org/October2021/#sec-Coercing-Field-Arguments>
pub(crate) fn coerce_argument_values(
    schema: &Schema,
    variable_values: &VariableValues,
    errors: &mut Vec<Error>,
    path: LinkedPath<'_>,
    field_def: &FieldDefinition,
    field: &Field,
) -> Result<JsonMap, PropagateNull> {
    let mut coerced_values = JsonMap::new();
    for arg_def in &field_def.arguments {
        let arg_name = &arg_def.name;
        if let Some(arg) = field.arguments.iter().find(|arg| arg.name == *arg_name) {
            if let Value::Variable(var_name) = arg.value.as_ref() {
                if let Some(var_value) = variable_values.get(var_name.as_str()) {
                    if var_value.is_null() && arg_def.ty.is_non_null() {
                        errors.push(field_error(
                            format!("null value for non-nullable argument {arg_name}"),
                            path,
                            arg_def.location(),
                        ));
                        return Err(PropagateNull);
                    } else {
                        coerced_values.insert(arg_name.as_str(), var_value.clone());
                        continue;
                    }
                }
            } else if arg.value.is_null() && arg_def.ty.is_non_null() {
                errors.push(field_error(
                    format!("null value for non-nullable argument {arg_name}"),
                    path,
                    arg_def.location(),
                ));
                return Err(PropagateNull);
            } else {
                let coerced_value = coerce_argument_value(
                    schema,
                    variable_values,
                    errors,
                    path,
                    "argument",
                    "",
                    "",
                    arg_name,
                    &arg_def.ty,
                    &arg.value,
                )?;
                coerced_values.insert(arg_name.as_str(), coerced_value);
                continue;
            }
        }
        if let Some(default) = &arg_def.default_value {
            let value =
                graphql_value_to_json("argument", "", "", arg_name, default).map_err(|err| {
                    errors.push(err.into_field_error(path, arg_def.location()));
                    PropagateNull
                })?;
            coerced_values.insert(arg_def.name.as_str(), value);
            continue;
        }
        if arg_def.ty.is_non_null() {
            errors.push(field_error(
                format!("missing value for required argument {arg_name}"),
                path,
                arg_def.location(),
            ));
            return Err(PropagateNull);
        }
    }
    Ok(coerced_values)
}

fn coerce_argument_value(
    schema: &Schema,
    variable_values: &VariableValues,
    errors: &mut Vec<Error>,
    path: LinkedPath<'_>,
    kind: &str,
    parent: &str,
    sep: &str,
    name: &str,
    ty: &Type,
    value: &Node<Value>,
) -> Result<JsonValue, PropagateNull> {
    if value.is_null() {
        if ty.is_non_null() {
            errors.push(field_error(
                format!("null value for non-null {kind} {parent}{sep}{name}"),
                path,
                value.location(),
            ));
            return Err(PropagateNull);
        } else {
            return Ok(JsonValue::Null);
        }
    }
    if let Some(var_name) = value.as_variable() {
        if let Some(var_value) = variable_values.get(var_name.as_str()) {
            if var_value.is_null() && ty.is_non_null() {
                errors.push(field_error(
                    format!("null variable value for non-null {kind} {parent}{sep}{name}"),
                    path,
                    value.location(),
                ));
                return Err(PropagateNull);
            } else {
                return Ok(var_value.clone());
            }
        } else if ty.is_non_null() {
            errors.push(field_error(
                format!("missing variable for non-null {kind} {parent}{sep}{name}"),
                path,
                value.location(),
            ));
            return Err(PropagateNull);
        } else {
            return Ok(JsonValue::Null);
        }
    }
    let ty_name = match ty {
        Type::List(inner_ty) | Type::NonNullList(inner_ty) => {
            // https://spec.graphql.org/October2021/#sec-List.Input-Coercion
            return value
                .as_list()
                // If not an array, treat the value as an array of size one:
                .unwrap_or(std::slice::from_ref(value))
                .iter()
                .map(|item| {
                    coerce_argument_value(
                        schema,
                        variable_values,
                        errors,
                        path,
                        kind,
                        parent,
                        sep,
                        name,
                        inner_ty,
                        item,
                    )
                })
                .collect();
        }
        Type::Named(ty_name) | Type::NonNullNamed(ty_name) => ty_name,
    };
    let Some(ty_def) = schema.types.get(ty_name) else {
        errors.push(
            field_error(
                format!("Undefined type {ty_name} for {kind} {parent}{sep}{name}"),
                path,
                value.location(),
            )
            .validation_should_have_caught_this(),
        );
        return Err(PropagateNull);
    };
    match ty_def {
        ExtendedType::InputObject(ty_def) => {
            // https://spec.graphql.org/October2021/#sec-Input-Objects.Input-Coercion
            if let Some(object) = value.as_object() {
                if let Some((key, _value)) = object
                    .iter()
                    .find(|(key, _value)| !ty_def.fields.contains_key(key))
                {
                    errors.push(field_error(
                        format!("Input object has key {key} not in type {ty_name}",),
                        path,
                        value.location(),
                    ));
                    return Err(PropagateNull);
                }
                // `map` converts `&(k, v)` to `(&k, &v)`
                let object: HashMap<_, _> = object.iter().map(|(k, v)| (k, v)).collect();
                let mut coerced_object = JsonMap::new();
                for (field_name, field_def) in &ty_def.fields {
                    if let Some(field_value) = object.get(field_name) {
                        let coerced_value = coerce_argument_value(
                            schema,
                            variable_values,
                            errors,
                            path,
                            "input field",
                            ty_name,
                            ".",
                            field_name,
                            &field_def.ty,
                            field_value,
                        )?;
                        coerced_object.insert(field_name.as_str(), coerced_value);
                    } else if let Some(default) = &field_def.default_value {
                        let default =
                            graphql_value_to_json("input field", ty_name, ".", field_name, default)
                                .map_err(|err| {
                                    errors.push(err.into_field_error(path, value.location()));
                                    PropagateNull
                                })?;
                        coerced_object.insert(field_name.as_str(), default);
                    } else if field_def.ty.is_non_null() {
                        errors.push(field_error(
                            format!(
                                "Missing value for non-null input object field {ty_name}.{field_name}"
                            ),
                            path,
                            value.location(),
                        ));
                        return Err(PropagateNull);
                    } else {
                        // Field not required
                    }
                }
                return Ok(coerced_object.into());
            }
        }
        _ => {
            // For scalar and enums, rely and validation and just convert between Rust types
            return graphql_value_to_json(kind, parent, sep, name, value).map_err(|err| {
                errors.push(err.into_field_error(path, value.location()));
                PropagateNull
            });
        }
    }
    errors.push(field_error(
        format!("Could not coerce {kind} {parent}{sep}{name}: {value} to type {ty_name}"),
        path,
        value.location(),
    ));
    Err(PropagateNull)
}
