use crate::execution::execute_selection_set;
use crate::execution::try_nullify;
use crate::execution::ExecutionMode;
use crate::execution::PropagateNull;
use crate::resolver::ResolvedValue;
use crate::response::field_error;
use crate::response::Error;
use crate::response::LinkedPath;
use crate::response::LinkedPathElement;
use crate::response::PathElement;
use crate::VariableValues;
use apollo_compiler::executable::Field;
use apollo_compiler::schema::ExtendedType;
use apollo_compiler::schema::Type;
use apollo_compiler::ExecutableDocument;
use apollo_compiler::Schema;
use futures::Stream;
use futures::StreamExt;
use serde_json_bytes::Value as JsonValue;

/// <https://spec.graphql.org/October2021/#CompleteValue()>
///
/// Returns `Err` for a field error being propagated upwards to find a nullable place
#[async_recursion::async_recursion]
pub(crate) async fn complete_value<'a, 'b>(
    schema: &'a Schema,
    document: &'a ExecutableDocument,
    variable_values: &'a VariableValues,
    errors: &'b mut Vec<Error>,
    path: LinkedPath<'b>,
    mode: ExecutionMode,
    ty: &'a Type,
    resolved: ResolvedValue<'a>,
    fields: &'a [&'a Field],
) -> Result<JsonValue, PropagateNull> {
    let new_field_error = |message| field_error(message, path, fields[0].name.location());
    macro_rules! field_error {
        ($($arg: tt)+) => {
            {
                errors.push(new_field_error(format!($($arg)+)));
                return Err(PropagateNull);
            }
        };
    }
    if let ResolvedValue::Leaf(JsonValue::Null) = resolved {
        if ty.is_non_null() {
            field_error!("Non-null type {ty} resolved to null")
        } else {
            return Ok(JsonValue::Null);
        }
    }
    if let ResolvedValue::List(stream) = resolved {
        match ty {
            Type::Named(_) | Type::NonNullNamed(_) => {
                field_error!("Non-list type {ty} resolved to a list")
            }
            Type::List(inner_ty) | Type::NonNullList(inner_ty) => {
                let mut completed_list = Vec::with_capacity(stream.size_hint().0);
                let mut stream = stream.enumerate();
                while let Some((index, inner_resolved)) = stream.next().await {
                    let inner_path = LinkedPathElement {
                        element: PathElement::ListItem { index },
                        next: path,
                    };
                    let inner_result = complete_value(
                        schema,
                        document,
                        variable_values,
                        errors,
                        Some(&inner_path),
                        mode,
                        inner_ty,
                        inner_resolved,
                        fields,
                    )
                    .await;
                    // On field error, try to nullify that item
                    match try_nullify(inner_ty, inner_result) {
                        Ok(inner_value) => completed_list.push(inner_value),
                        // If the item is non-null, try to nullify the list
                        Err(PropagateNull) => return try_nullify(ty, Err(PropagateNull)),
                    }
                }
                return Ok(completed_list.into());
            }
        }
    }
    let ty_name = match ty {
        Type::List(_) | Type::NonNullList(_) => {
            field_error!("List type {ty} resolved to an object")
        }
        Type::Named(name) | Type::NonNullNamed(name) => name,
    };
    let Some(ty_def) = schema.types.get(ty_name) else {
        errors.push(
            new_field_error(format!("Undefined type {ty_name}"))
                .validation_should_have_caught_this(),
        );
        return Err(PropagateNull);
    };
    if let ExtendedType::InputObject(_) = ty_def {
        errors.push(
            new_field_error(format!("Field with input object type {ty_name}"))
                .validation_should_have_caught_this(),
        );
        return Err(PropagateNull);
    }
    let resolved_obj = match resolved {
        ResolvedValue::List(_) => unreachable!(), // early return above
        ResolvedValue::Leaf(json_value) => {
            match ty_def {
                ExtendedType::InputObject(_) => unreachable!(), // early return above
                ExtendedType::Object(_) | ExtendedType::Interface(_) | ExtendedType::Union(_) => {
                    field_error!(
                        "Resolver returned a leaf value \
                         but expected an object for type {ty_name}"
                    )
                }
                ExtendedType::Enum(enum_def) => {
                    // https://spec.graphql.org/October2021/#sec-Enums.Result-Coercion
                    if !json_value
                        .as_str()
                        .is_some_and(|str| enum_def.values.contains_key(str))
                    {
                        field_error!("Resolver returned {json_value}, expected enum {ty_name}")
                    }
                }
                ExtendedType::Scalar(_) => match ty_name.as_str() {
                    "Int" => {
                        // https://spec.graphql.org/October2021/#sec-Int.Result-Coercion
                        // > GraphQL services may coerce non-integer internal values to integers
                        // > when reasonable without losing information
                        //
                        // We choose not to, to keep with Rust’s strong typing
                        if let Some(int) = json_value.as_i64() {
                            if i32::try_from(int).is_err() {
                                field_error!("Resolver returned {json_value} which overflows Int")
                            }
                        } else {
                            field_error!("Resolver returned {json_value}, expected Int")
                        }
                    }
                    "Float" => {
                        // https://spec.graphql.org/October2021/#sec-Float.Result-Coercion
                        if !json_value.is_f64() {
                            field_error!("Resolver returned {json_value}, expected Float")
                        }
                    }
                    "String" => {
                        // https://spec.graphql.org/October2021/#sec-String.Result-Coercion
                        if !json_value.is_string() {
                            field_error!("Resolver returned {json_value}, expected String")
                        }
                    }
                    "Boolean" => {
                        // https://spec.graphql.org/October2021/#sec-Boolean.Result-Coercion
                        if !json_value.is_boolean() {
                            field_error!("Resolver returned {json_value}, expected Boolean")
                        }
                    }
                    "ID" => {
                        // https://spec.graphql.org/October2021/#sec-ID.Result-Coercion
                        if !(json_value.is_string() || json_value.is_i64()) {
                            field_error!("Resolver returned {json_value}, expected ID")
                        }
                    }
                    _ => {
                        // Custom scalar: accept any JSON value (including an array or object,
                        // despite this being a "leaf" as far as GraphQL resolution is concerned)
                    }
                },
            };
            return Ok(json_value);
        }
        ResolvedValue::Object(resolved_obj) => resolved_obj,
    };
    let (object_type_name, object_type) = match ty_def {
        ExtendedType::InputObject(_) => unreachable!(), // early return above
        ExtendedType::Enum(_) | ExtendedType::Scalar(_) => {
            field_error!(
                "Resolver returned a an object of type {}, expected {ty_name}",
                resolved_obj.type_name()
            )
        }
        ExtendedType::Interface(_) | ExtendedType::Union(_) => {
            let object_type_name = resolved_obj.type_name();
            if let Some(def) = schema.get_object(object_type_name) {
                (object_type_name, def)
            } else {
                field_error!(
                    "Resolver returned an object of type {object_type_name} \
                     not defined in the schema"
                )
            }
        }
        ExtendedType::Object(def) => {
            debug_assert_eq!(ty_name, resolved_obj.type_name());
            (ty_name.as_str(), def)
        }
    };
    execute_selection_set(
        schema,
        document,
        variable_values,
        errors,
        path,
        mode,
        object_type_name,
        object_type,
        &*resolved_obj,
        fields
            .iter()
            .flat_map(|field| &field.selection_set.selections),
    )
    .await
    .map(JsonValue::Object)
}
