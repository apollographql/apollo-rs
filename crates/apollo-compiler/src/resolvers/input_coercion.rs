use crate::ast::Type;
use crate::ast::Value;
use crate::collections::HashMap;
use crate::executable::Field;
use crate::executable::Operation;
use crate::parser::SourceMap;
use crate::parser::SourceSpan;
use crate::resolvers::execution::ExecutionContext;
use crate::resolvers::execution::LinkedPath;
use crate::resolvers::execution::PropagateNull;
use crate::response::GraphQLError;
use crate::response::JsonMap;
use crate::response::JsonValue;
use crate::schema::ExtendedType;
use crate::schema::FieldDefinition;
use crate::validation::SuspectedValidationBug;
use crate::validation::Valid;
use crate::Node;
use crate::Schema;

/// The maximum integer safely representable as an IEEE 754 double-precision float.
const MAX_SAFE_INT: i64 = (1_i64 << 53) - 1;

#[derive(Debug, Clone)]
pub(crate) enum InputCoercionError {
    SuspectedValidationBug(SuspectedValidationBug),
    // TODO: split into more structured variants?
    ValueError {
        message: String,
        location: Option<SourceSpan>,
    },
}

// Documented in `src/request.rs`
pub(crate) fn coerce_variable_values(
    schema: &Valid<Schema>,
    operation: &Operation,
    values: &JsonMap,
) -> Result<Valid<JsonMap>, InputCoercionError> {
    let mut coerced_values = JsonMap::new();
    for variable_def in &operation.variables {
        let name = variable_def.name.as_str();
        if let Some((key, value)) = values.get_key_value(name) {
            let value = coerce_variable_value(
                schema,
                &format_args!("variable {name}"),
                &variable_def.ty,
                value,
            )?;
            coerced_values.insert(key.clone(), value);
        } else if let Some(default) = &variable_def.default_value {
            // https://spec.graphql.org/September2025/#sec-Coercing-Variable-Values
            // > Let coercedDefaultValue be the result of coercing defaultValue
            // > according to the input coercion rules of variableType.
            let value = coerce_default_value(
                schema,
                &format_args!("default value of variable {name}"),
                &variable_def.ty,
                default,
            )?;
            coerced_values.insert(name, value);
        } else if variable_def.ty.is_non_null() {
            return Err(InputCoercionError::ValueError {
                message: format!("missing value for non-null variable '{name}'"),
                location: variable_def.location(),
            });
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

/// As of September2025 default values are coerced like any other input value
/// instead of being used verbatim, so that (for example) the defaults of
/// nested input object fields omitted from a default value are applied.
fn coerce_default_value(
    schema: &Valid<Schema>,
    description: &std::fmt::Arguments<'_>,
    ty: &Type,
    default: &Node<Value>,
) -> Result<JsonValue, InputCoercionError> {
    let value = graphql_value_to_json(description, default)?;
    coerce_variable_value(schema, description, ty, &value)
}

fn coerce_variable_value(
    schema: &Valid<Schema>,
    description: &std::fmt::Arguments<'_>,
    ty: &Type,
    value: &JsonValue,
) -> Result<JsonValue, InputCoercionError> {
    if value.is_null() {
        if ty.is_non_null() {
            return Err(InputCoercionError::ValueError {
                message: format!("null value for {description} of non-null type {ty}"),
                location: None,
            });
        } else {
            return Ok(JsonValue::Null);
        }
    }
    let ty_name = match ty {
        Type::List(inner) | Type::NonNullList(inner) => {
            // https://spec.graphql.org/September2025/#sec-List.Input-Coercion
            return value
                .as_array()
                .map(Vec::as_slice)
                // If not an array, treat the value as an array of size one:
                .unwrap_or(std::slice::from_ref(value))
                .iter()
                .map(|item| coerce_variable_value(schema, description, inner, item))
                .collect();
        }
        Type::Named(ty_name) | Type::NonNullNamed(ty_name) => ty_name,
    };
    let Some(ty_def) = schema.types.get(ty_name) else {
        Err(SuspectedValidationBug {
            message: format!("undefined type {ty_name} for {description}"),
            location: ty_name.location(),
        })?
    };
    match ty_def {
        ExtendedType::Object(_) | ExtendedType::Interface(_) | ExtendedType::Union(_) => {
            Err(SuspectedValidationBug {
                message: format!("non-input type {ty_name} for {description}."),
                location: ty_name.location(),
            })?
        }
        ExtendedType::Scalar(_) => match ty_name.as_str() {
            "Int" => {
                // https://spec.graphql.org/September2025/#sec-Int.Input-Coercion
                if value
                    .as_i64()
                    .is_some_and(|value| i32::try_from(value).is_ok())
                {
                    return Ok(value.clone());
                }
            }
            "Float" => {
                // https://spec.graphql.org/September2025/#sec-Float.Input-Coercion
                if value.is_f64()
                    || value
                        .as_f64()
                        .is_some_and(|f| f.abs() <= MAX_SAFE_INT as f64)
                {
                    return Ok(value.clone());
                }
            }
            "String" => {
                // https://spec.graphql.org/September2025/#sec-String.Input-Coercion
                if value.is_string() {
                    return Ok(value.clone());
                }
            }
            "Boolean" => {
                // https://spec.graphql.org/September2025/#sec-Boolean.Input-Coercion
                if value.is_boolean() {
                    return Ok(value.clone());
                }
            }
            "ID" => {
                // https://spec.graphql.org/September2025/#sec-ID.Input-Coercion
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
            // https://spec.graphql.org/September2025/#sec-Enums.Input-Coercion
            if let Some(str) = value.as_str() {
                if ty_def.values.keys().any(|value_name| value_name == str) {
                    return Ok(value.clone());
                }
            }
        }
        ExtendedType::InputObject(ty_def) => {
            // https://spec.graphql.org/September2025/#sec-Input-Objects.Input-Coercion
            if let Some(object) = value.as_object() {
                if let Some(key) = object
                    .keys()
                    .find(|key| !ty_def.fields.contains_key(key.as_str()))
                {
                    return Err(InputCoercionError::ValueError {
                        message: format!(
                            "Input object has key {} not in type {ty_name}",
                            key.as_str()
                        ),
                        location: None,
                    });
                }
                // @oneOf pre-coercion: the provided value must contain exactly one
                // entry and that entry must not be null.
                // https://spec.graphql.org/September2025/#sec-OneOf-Input-Objects.Input-Coercion
                if ty_def.is_one_of() {
                    let provided_count = object
                        .keys()
                        .filter(|k| ty_def.fields.contains_key(k.as_str()))
                        .count();
                    if provided_count != 1 {
                        return Err(InputCoercionError::ValueError {
                            message: format!(
                                "@oneOf input object '{ty_name}' must specify exactly one key, \
                                 but {provided_count} were given",
                            ),
                            location: None,
                        });
                    }
                    if let Some((field_name, field_value)) = object.iter().next() {
                        if field_value.is_null() {
                            return Err(InputCoercionError::ValueError {
                                message: format!(
                                    "@oneOf input object '{ty_name}' field '{}' \
                                     must be non-null",
                                    field_name.as_str()
                                ),
                                location: None,
                            });
                        }
                    }
                }
                let mut object = object.clone();
                for (field_name, field_def) in &ty_def.fields {
                    if let Some(field_value) = object.get_mut(field_name.as_str()) {
                        *field_value = coerce_variable_value(
                            schema,
                            &format_args!("input field {ty_name}.{field_name}"),
                            &field_def.ty,
                            field_value,
                        )?
                    } else if let Some(default) = &field_def.default_value {
                        let default = coerce_default_value(
                            schema,
                            &format_args!("input field {ty_name}.{field_name}"),
                            &field_def.ty,
                            default,
                        )?;
                        object.insert(field_name.as_str(), default);
                    } else if field_def.ty.is_non_null() {
                        return Err(InputCoercionError::ValueError {
                            message: format!("Missing value for non-null input object field {ty_name}.{field_name}"),
                            location: None,
                        });
                    } else {
                        // Field not required
                    }
                }
                // @oneOf post-coercion: the resulting coerced map must contain
                // exactly one entry whose value is not null.
                // https://spec.graphql.org/September2025/#sec-OneOf-Input-Objects.Input-Coercion
                if ty_def.is_one_of() {
                    let non_null_count = object.iter().filter(|(_, v)| !v.is_null()).count();
                    if non_null_count != 1 {
                        return Err(InputCoercionError::ValueError {
                            message: format!(
                                "@oneOf input object '{ty_name}' must have exactly one non-null \
                                 field after coercion, but {non_null_count} were given",
                            ),
                            location: None,
                        });
                    }
                }
                return Ok(object.into());
            }
        }
    }
    Err(InputCoercionError::ValueError {
        message: format!("could not coerce {description}: {value} to type {ty_name}"),
        location: None,
    })
}

/// Converts a constant GraphQL value (like a default value) to JSON.
///
/// Values from a `Valid` document do not contain variables in const positions.
fn graphql_value_to_json(
    description: &std::fmt::Arguments<'_>,
    value: &Node<Value>,
) -> Result<JsonValue, InputCoercionError> {
    graphql_literal_to_json(description, value, None)
}

/// Converts a GraphQL value to JSON.
///
/// `variable_values` is `Some` when converting an argument literal from an
/// executable document, where a custom scalar value may contain nested variables
/// to substitute with their runtime values.
/// It is `None` for const values (like default values), where validation
/// guarantees the absence of variables.
fn graphql_literal_to_json(
    description: &std::fmt::Arguments<'_>,
    value: &Node<Value>,
    variable_values: Option<&Valid<JsonMap>>,
) -> Result<JsonValue, InputCoercionError> {
    match value.as_ref() {
        Value::Null => Ok(JsonValue::Null),
        Value::Variable(var_name) => {
            if let Some(variable_values) = variable_values {
                // A variable nested inside a custom scalar literal is replaced
                // with its runtime value; with no runtime value it becomes null
                // (like a missing list item in graphql-js).
                Ok(variable_values
                    .get(var_name.as_str())
                    .cloned()
                    .unwrap_or(JsonValue::Null))
            } else {
                // TODO: separate `ContValue` enum without this variant?
                Err(InputCoercionError::SuspectedValidationBug(
                    SuspectedValidationBug {
                        message: format!("variable in default value of {description}."),
                        location: value.location(),
                    },
                ))
            }
        }
        Value::Enum(value) => Ok(value.as_str().into()),
        Value::String(value) => Ok(value.as_str().into()),
        Value::Boolean(value) => Ok((*value).into()),
        // Rely on `serde_json::Number`’s own parser to use whatever precision it supports
        Value::Int(i) => Ok(JsonValue::Number(i.as_str().parse().map_err(|_| {
            InputCoercionError::ValueError {
                message: format!("int value overflow in {description}"),
                location: value.location(),
            }
        })?)),
        Value::Float(f) => Ok(JsonValue::Number(f.as_str().parse().map_err(|_| {
            InputCoercionError::ValueError {
                message: format!("float value overflow in {description}"),
                location: value.location(),
            }
        })?)),
        Value::List(value) => value
            .iter()
            .map(|value| graphql_literal_to_json(description, value, variable_values))
            .collect(),
        Value::Object(value) => value
            .iter()
            .filter(|(_key, value)| {
                // An entry whose value is a variable with no runtime value
                // is omitted entirely, like in graphql-js
                match (value.as_ref(), variable_values) {
                    (Value::Variable(var_name), Some(variable_values)) => {
                        variable_values.contains_key(var_name.as_str())
                    }
                    _ => true,
                }
            })
            .map(|(key, value)| {
                Ok((
                    key.as_str(),
                    graphql_literal_to_json(description, value, variable_values)?,
                ))
            })
            .collect(),
    }
}

/// <https://spec.graphql.org/September2025/#sec-Coercing-Field-Arguments>
pub(crate) fn coerce_argument_values(
    ctx: &mut ExecutionContext<'_>,
    path: LinkedPath<'_>,
    field_def: &FieldDefinition,
    field: &Field,
) -> Result<JsonMap, PropagateNull> {
    let mut coerced_values = JsonMap::new();
    for arg_def in &field_def.arguments {
        let arg_name = &arg_def.name;
        if let Some(arg) = field.arguments.iter().find(|arg| arg.name == *arg_name) {
            if let Value::Variable(var_name) = arg.value.as_ref() {
                if let Some(var_value) = ctx.variable_values.get(var_name.as_str()) {
                    if var_value.is_null() && arg_def.ty.is_non_null() {
                        ctx.errors.push(GraphQLError::execution_error(
                            format!("null value for non-nullable argument {arg_name}"),
                            path,
                            arg_def.location(),
                            &ctx.document.sources,
                        ));
                        return Err(PropagateNull);
                    } else {
                        coerced_values.insert(arg_name.as_str(), var_value.clone());
                        continue;
                    }
                }
            } else if arg.value.is_null() && arg_def.ty.is_non_null() {
                ctx.errors.push(GraphQLError::execution_error(
                    format!("null value for non-nullable argument {arg_name}"),
                    path,
                    arg_def.location(),
                    &ctx.document.sources,
                ));
                return Err(PropagateNull);
            } else {
                let coerced_value = coerce_argument_value(
                    ctx,
                    path,
                    &format_args!("argument {arg_name}"),
                    &arg_def.ty,
                    &arg.value,
                )?;
                coerced_values.insert(arg_name.as_str(), coerced_value);
                continue;
            }
        }
        if let Some(default) = &arg_def.default_value {
            // https://spec.graphql.org/September2025/#sec-Coercing-Field-Arguments
            // > Let coercedDefaultValue be the result of coercing defaultValue
            // > according to the input coercion rules of argumentType.
            // > Any request error raised as a result of input coercion during
            // > CoerceArgumentValues() should be treated instead as an execution error.
            let value = coerce_default_value(
                ctx.schema,
                &format_args!("argument {arg_name}"),
                &arg_def.ty,
                default,
            )
            .map_err(|err| {
                ctx.errors
                    .push(err.into_execution_error(path, &ctx.document.sources));
                PropagateNull
            })?;
            coerced_values.insert(arg_def.name.as_str(), value);
            continue;
        }
        if arg_def.ty.is_non_null() {
            ctx.errors.push(GraphQLError::execution_error(
                format!("missing value for required argument {arg_name}"),
                path,
                arg_def.location(),
                &ctx.document.sources,
            ));
            return Err(PropagateNull);
        }
    }
    Ok(coerced_values)
}

fn coerce_argument_value(
    ctx: &mut ExecutionContext<'_>,
    path: LinkedPath<'_>,
    description: &std::fmt::Arguments<'_>,
    ty: &Type,
    value: &Node<Value>,
) -> Result<JsonValue, PropagateNull> {
    if value.is_null() {
        if ty.is_non_null() {
            ctx.errors.push(GraphQLError::execution_error(
                format!("null value for non-null {description}"),
                path,
                value.location(),
                &ctx.document.sources,
            ));
            return Err(PropagateNull);
        } else {
            return Ok(JsonValue::Null);
        }
    }
    if let Some(var_name) = value.as_variable() {
        if let Some(var_value) = ctx.variable_values.get(var_name.as_str()) {
            if var_value.is_null() && ty.is_non_null() {
                ctx.errors.push(GraphQLError::execution_error(
                    format!("null variable value for non-null {description}"),
                    path,
                    value.location(),
                    &ctx.document.sources,
                ));
                return Err(PropagateNull);
            } else {
                return Ok(var_value.clone());
            }
        } else if ty.is_non_null() {
            ctx.errors.push(GraphQLError::execution_error(
                format!("missing variable for non-null {description}"),
                path,
                value.location(),
                &ctx.document.sources,
            ));
            return Err(PropagateNull);
        } else {
            return Ok(JsonValue::Null);
        }
    }
    let ty_name = match ty {
        Type::List(inner_ty) | Type::NonNullList(inner_ty) => {
            // https://spec.graphql.org/September2025/#sec-List.Input-Coercion
            return value
                .as_list()
                // If not an array, treat the value as an array of size one:
                .unwrap_or(std::slice::from_ref(value))
                .iter()
                .map(|item| coerce_argument_value(ctx, path, description, inner_ty, item))
                .collect();
        }
        Type::Named(ty_name) | Type::NonNullNamed(ty_name) => ty_name,
    };
    let Some(ty_def) = ctx.schema.types.get(ty_name) else {
        ctx.errors.push(
            SuspectedValidationBug {
                message: format!("undefined type {ty_name} for {description}"),
                location: value.location(),
            }
            .into_execution_error(&ctx.document.sources, path),
        );
        return Err(PropagateNull);
    };
    match ty_def {
        ExtendedType::InputObject(ty_def) => {
            // https://spec.graphql.org/September2025/#sec-Input-Objects.Input-Coercion
            if let Some(object) = value.as_object() {
                if let Some((key, _value)) = object
                    .iter()
                    .find(|(key, _value)| !ty_def.fields.contains_key(key.as_str()))
                {
                    ctx.errors.push(GraphQLError::execution_error(
                        format!("input object has key {key} not in type {ty_name}",),
                        path,
                        value.location(),
                        &ctx.document.sources,
                    ));
                    return Err(PropagateNull);
                }
                // @oneOf pre-coercion: the provided value must contain exactly one
                // entry and that entry must not be null.
                // https://spec.graphql.org/September2025/#sec-OneOf-Input-Objects.Input-Coercion
                if ty_def.is_one_of() {
                    let provided_count = object
                        .iter()
                        .filter(|(k, _)| ty_def.fields.contains_key(k.as_str()))
                        .count();
                    if provided_count != 1 {
                        ctx.errors.push(GraphQLError::execution_error(
                            format!(
                                "@oneOf input object '{ty_name}' must specify exactly one key, \
                                 but {provided_count} were given",
                            ),
                            path,
                            value.location(),
                            &ctx.document.sources,
                        ));
                        return Err(PropagateNull);
                    }
                    if let Some((field_name, field_value)) = object.iter().next() {
                        if field_value.is_null() {
                            ctx.errors.push(GraphQLError::execution_error(
                                format!(
                                    "@oneOf input object '{ty_name}' field '{field_name}' \
                                     must be non-null"
                                ),
                                path,
                                value.location(),
                                &ctx.document.sources,
                            ));
                            return Err(PropagateNull);
                        }
                    }
                }
                #[allow(clippy::map_identity)] // `map` converts `&(k, v)` to `(&k, &v)`
                let object: HashMap<_, _> = object.iter().map(|(k, v)| (k, v)).collect();
                let mut coerced_object = JsonMap::new();
                for (field_name, field_def) in &ty_def.fields {
                    // A field whose value is a variable with no runtime value
                    // behaves as if the field was not provided at all:
                    // fall back to the field’s default value, or omit the entry.
                    // https://spec.graphql.org/September2025/#sec-Input-Objects.Input-Coercion
                    let provided_value = object.get(field_name).copied().filter(|value| {
                        value
                            .as_variable()
                            .is_none_or(|var| ctx.variable_values.contains_key(var.as_str()))
                    });
                    if let Some(field_value) = provided_value {
                        let coerced_value = coerce_argument_value(
                            ctx,
                            path,
                            &format_args!("input field {ty_name}.{field_name}"),
                            &field_def.ty,
                            field_value,
                        )?;
                        coerced_object.insert(field_name.as_str(), coerced_value);
                    } else if let Some(default) = &field_def.default_value {
                        let default = coerce_default_value(
                            ctx.schema,
                            &format_args!("input field {ty_name}.{field_name}"),
                            &field_def.ty,
                            default,
                        )
                        .map_err(|err| {
                            ctx.errors
                                .push(err.into_execution_error(path, &ctx.document.sources));
                            PropagateNull
                        })?;
                        coerced_object.insert(field_name.as_str(), default);
                    } else if field_def.ty.is_non_null() {
                        ctx.errors.push(GraphQLError::execution_error(
                            format!(
                                "Missing value for non-null input object field {ty_name}.{field_name}"
                            ),
                            path,
                            value.location(),
                            &ctx.document.sources,
                        ));
                        return Err(PropagateNull);
                    } else {
                        // Field not required
                    }
                }
                // @oneOf post-coercion: the resulting coerced map must contain
                // exactly one entry whose value is not null.
                // https://spec.graphql.org/September2025/#sec-OneOf-Input-Objects.Input-Coercion
                if ty_def.is_one_of() {
                    let non_null_count =
                        coerced_object.iter().filter(|(_, v)| !v.is_null()).count();
                    if non_null_count != 1 {
                        ctx.errors.push(GraphQLError::execution_error(
                            format!(
                                "@oneOf input object '{ty_name}' must have exactly one non-null \
                                 field, but {non_null_count} {} given",
                                if non_null_count == 1 { "was" } else { "were" }
                            ),
                            path,
                            value.location(),
                            &ctx.document.sources,
                        ));
                        return Err(PropagateNull);
                    }
                }
                return Ok(coerced_object.into());
            }
        }
        _ => {
            // For scalars and enums, rely on validation and just convert between Rust types,
            // substituting any variables nested inside custom scalar literals
            return graphql_literal_to_json(description, value, Some(ctx.variable_values)).map_err(
                |err| {
                    ctx.errors
                        .push(err.into_execution_error(path, &ctx.document.sources));
                    PropagateNull
                },
            );
        }
    }
    ctx.errors.push(GraphQLError::execution_error(
        format!("could not coerce {description}: {value} to type {ty_name}"),
        path,
        value.location(),
        &ctx.document.sources,
    ));
    Err(PropagateNull)
}

impl From<SuspectedValidationBug> for InputCoercionError {
    fn from(value: SuspectedValidationBug) -> Self {
        Self::SuspectedValidationBug(value)
    }
}

impl InputCoercionError {
    pub(crate) fn into_execution_error(
        self,
        path: LinkedPath<'_>,
        sources: &SourceMap,
    ) -> GraphQLError {
        match self {
            Self::SuspectedValidationBug(s) => s.into_execution_error(sources, path),
            Self::ValueError { message, location } => {
                GraphQLError::execution_error(message, path, location, sources)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::validation::Valid;
    use crate::ExecutableDocument;
    use crate::Schema;

    fn schema_and_doc_with_float_arg() -> (Valid<Schema>, Valid<ExecutableDocument>) {
        let schema = Schema::parse_and_validate(
            r#"
                type Query {
                    foo(bar: Float!): Float!
                }
            "#,
            "sdl",
        )
        .unwrap();
        let doc = ExecutableDocument::parse_and_validate(
            &schema,
            "query ($bar: Float!) { foo(bar: $bar) }",
            "op.graphql",
        )
        .unwrap();
        (schema, doc)
    }

    #[test]
    fn coerces_float_to_float() {
        let float_beyond_integer_max = (MAX_SAFE_INT as f64) + 0.5;
        let variables = serde_json_bytes::json!({ "bar": float_beyond_integer_max });
        let (schema, doc) = schema_and_doc_with_float_arg();

        // When a float greater than MAX_SAFE_INT is provided, it should be accepted as a float.
        let _ = coerce_variable_values(
            &schema,
            doc.operations.anonymous.as_ref().unwrap(),
            variables.as_object().unwrap(),
        )
        .unwrap();
    }

    #[test]
    fn coerces_int_to_float() {
        let variables = serde_json_bytes::json!({ "bar": 14 });
        let (schema, doc) = schema_and_doc_with_float_arg();

        // When an integer within the safe bounds is provided, it should be accepted as a float.
        let _ = coerce_variable_values(
            &schema,
            doc.operations.anonymous.as_ref().unwrap(),
            variables.as_object().unwrap(),
        )
        .unwrap();
    }

    #[test]
    fn fails_to_coerce_int_to_float_beyond_precision_bound() {
        let variables = serde_json_bytes::json!({ "bar": i64::MAX });
        let (schema, doc) = schema_and_doc_with_float_arg();

        // When an integer cannot be finitely represented as a float, it should be rejected.
        let _ = coerce_variable_values(
            &schema,
            doc.operations.anonymous.as_ref().unwrap(),
            variables.as_object().unwrap(),
        )
        .unwrap_err();
    }

    #[test]
    fn fails_to_numeric_string_to_float() {
        let variables = serde_json_bytes::json!({ "bar": "14" });
        let (schema, doc) = schema_and_doc_with_float_arg();

        // Strings (even numeric ones) should not be coerced to Float in input positions.
        let _ = coerce_variable_values(
            &schema,
            doc.operations.anonymous.as_ref().unwrap(),
            variables.as_object().unwrap(),
        )
        .unwrap_err();
    }

    #[test]
    fn fails_to_coerce_inf_to_float() {
        let variables = serde_json_bytes::json!({ "bar": f64::INFINITY });
        let (schema, doc) = schema_and_doc_with_float_arg();

        // Infinity should not be accepted as a Float input value.
        let _ = coerce_variable_values(
            &schema,
            doc.operations.anonymous.as_ref().unwrap(),
            variables.as_object().unwrap(),
        )
        .unwrap_err();
    }

    #[test]
    fn fails_to_coerce_nan_to_float() {
        let variables = serde_json_bytes::json!({ "bar": f64::NAN });
        let (schema, doc) = schema_and_doc_with_float_arg();

        // NaN should not be accepted as a Float input value.
        let _ = coerce_variable_values(
            &schema,
            doc.operations.anonymous.as_ref().unwrap(),
            variables.as_object().unwrap(),
        )
        .unwrap_err();
    }

    // -----------------------------------------------------------------------
    // @oneOf runtime coercion tests
    // https://spec.graphql.org/September2025/#sec-OneOf-Input-Objects
    // -----------------------------------------------------------------------

    fn one_of_schema_and_doc() -> (Valid<Schema>, Valid<ExecutableDocument>) {
        let schema = Schema::parse_and_validate(
            r#"
                type Query {
                    search(filter: SearchFilter): String
                }
                input SearchFilter @oneOf {
                    byName: String
                    byId: Int
                }
            "#,
            "schema.graphql",
        )
        .unwrap();
        let doc = ExecutableDocument::parse_and_validate(
            &schema,
            "query ($filter: SearchFilter) { search(filter: $filter) }",
            "op.graphql",
        )
        .unwrap();
        (schema, doc)
    }

    #[test]
    fn one_of_coercion_valid_single_field() {
        let (schema, doc) = one_of_schema_and_doc();
        let variables = serde_json_bytes::json!({ "filter": { "byName": "alice" } });
        coerce_variable_values(
            &schema,
            doc.operations.anonymous.as_ref().unwrap(),
            variables.as_object().unwrap(),
        )
        .expect("single non-null field should be accepted");
    }

    fn one_of_error_message(err: InputCoercionError) -> String {
        match err {
            InputCoercionError::ValueError { message, .. } => message,
            InputCoercionError::SuspectedValidationBug(b) => b.message,
        }
    }

    #[test]
    fn one_of_coercion_rejects_zero_fields() {
        let (schema, doc) = one_of_schema_and_doc();
        let variables = serde_json_bytes::json!({ "filter": {} });
        let err = coerce_variable_values(
            &schema,
            doc.operations.anonymous.as_ref().unwrap(),
            variables.as_object().unwrap(),
        )
        .unwrap_err();
        let msg = one_of_error_message(err);
        assert!(
            msg.contains("must specify exactly one key"),
            "unexpected error: {msg}"
        );
    }

    #[test]
    fn one_of_coercion_rejects_multiple_fields() {
        let (schema, doc) = one_of_schema_and_doc();
        let variables = serde_json_bytes::json!({ "filter": { "byName": "alice", "byId": 1 } });
        let err = coerce_variable_values(
            &schema,
            doc.operations.anonymous.as_ref().unwrap(),
            variables.as_object().unwrap(),
        )
        .unwrap_err();
        let msg = one_of_error_message(err);
        assert!(
            msg.contains("must specify exactly one key"),
            "unexpected error: {msg}"
        );
    }

    #[test]
    fn one_of_coercion_rejects_null_field_value() {
        let (schema, doc) = one_of_schema_and_doc();
        let variables = serde_json_bytes::json!({ "filter": { "byName": null } });
        let err = coerce_variable_values(
            &schema,
            doc.operations.anonymous.as_ref().unwrap(),
            variables.as_object().unwrap(),
        )
        .unwrap_err();
        let msg = one_of_error_message(err);
        assert!(msg.contains("must be non-null"), "unexpected error: {msg}");
    }
}
