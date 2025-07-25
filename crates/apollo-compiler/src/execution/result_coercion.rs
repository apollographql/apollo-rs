use crate::executable::Field;
use crate::execution::engine::execute_selection_set;
use crate::execution::engine::try_nullify;
use crate::execution::engine::ExecutionContext;
use crate::execution::engine::ExecutionMode;
use crate::execution::engine::LinkedPath;
use crate::execution::engine::LinkedPathElement;
use crate::execution::engine::PropagateNull;
use crate::execution::resolver::ResolvedValue;
use crate::response::GraphQLError;
use crate::response::JsonValue;
use crate::response::ResponseDataPathSegment;
use crate::schema::ExtendedType;
use crate::schema::Type;
use crate::validation::SuspectedValidationBug;

/// <https://spec.graphql.org/October2021/#CompleteValue()>
///
/// Returns `Err` for a field error being propagated upwards to find a nullable place
pub(crate) fn complete_value<'a>(
    ctx: &mut ExecutionContext<'a>,
    path: LinkedPath<'_>,
    mode: ExecutionMode,
    ty: &'a Type,
    resolved: ResolvedValue<'_>,
    fields: &[&'a Field],
) -> Result<JsonValue, PropagateNull> {
    let location = fields[0].name.location();
    macro_rules! field_error {
        ($($arg: tt)+) => {
            {
                ctx.errors.push(GraphQLError::field_error(
                    format!($($arg)+),
                    path,
                    location,
                    &ctx.document.sources
                ));
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
    if let ResolvedValue::List(iter) = resolved {
        match ty {
            Type::Named(_) | Type::NonNullNamed(_) => {
                field_error!("Non-list type {ty} resolved to a list")
            }
            Type::List(inner_ty) | Type::NonNullList(inner_ty) => {
                let mut completed_list = Vec::with_capacity(iter.size_hint().0);
                for (index, inner_result) in iter.enumerate() {
                    let inner_path = LinkedPathElement {
                        element: ResponseDataPathSegment::ListIndex(index),
                        next: path,
                    };
                    let inner_resolved = inner_result.map_err(|err| {
                        ctx.errors.push(GraphQLError::field_error(
                            format!("resolver error: {}", err.message),
                            Some(&inner_path),
                            fields[0].name.location(),
                            &ctx.document.sources,
                        ));
                        PropagateNull
                    })?;
                    let inner_result = complete_value(
                        ctx,
                        Some(&inner_path),
                        mode,
                        inner_ty,
                        inner_resolved,
                        fields,
                    );
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
    let Some(ty_def) = ctx.schema.types.get(ty_name) else {
        ctx.errors.push(
            SuspectedValidationBug {
                message: format!("Undefined type {ty_name}"),
                location,
            }
            .into_field_error(&ctx.document.sources, path),
        );
        return Err(PropagateNull);
    };
    if let ExtendedType::InputObject(_) = ty_def {
        ctx.errors.push(
            SuspectedValidationBug {
                message: format!("Field with input object type {ty_name}"),
                location,
            }
            .into_field_error(&ctx.document.sources, path),
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
    let object_type = match ty_def {
        ExtendedType::InputObject(_) => unreachable!(), // early return above
        ExtendedType::Enum(_) | ExtendedType::Scalar(_) => {
            field_error!(
                "Resolver returned a an object of type {}, expected {ty_name}",
                resolved_obj.type_name()
            )
        }
        ExtendedType::Interface(_) | ExtendedType::Union(_) => {
            let object_type_name = resolved_obj.type_name();
            if let Some(def) = ctx.schema.get_object(object_type_name) {
                def
            } else {
                field_error!(
                    "Resolver returned an object of type {object_type_name} \
                     not defined in the schema"
                )
            }
        }
        ExtendedType::Object(def) => {
            debug_assert_eq!(ty_name, resolved_obj.type_name());
            def
        }
    };
    execute_selection_set(
        ctx,
        path,
        mode,
        object_type,
        Some(&*resolved_obj),
        fields
            .iter()
            .flat_map(|field| &field.selection_set.selections),
    )
    .map(JsonValue::Object)
}

#[test]
fn test_error_path() {
    use super::resolver;
    use crate::introspection::resolvers::MaybeLazy;
    use crate::response::JsonMap;
    use crate::ExecutableDocument;
    use crate::Schema;
    use std::cell::OnceCell;

    let sdl = "type Query { f: [Int] }";
    let query = "{ f }";

    struct InitialValue;

    impl resolver::ObjectValue for InitialValue {
        fn type_name(&self) -> &str {
            "Query"
        }

        fn resolve_field<'a>(
            &'a self,
            field_name: &'a str,
            _arguments: &'a JsonMap,
        ) -> Result<ResolvedValue<'a>, resolver::ResolveError> {
            match field_name {
                "f" => Ok(ResolvedValue::List(Box::new(
                    [
                        Ok(ResolvedValue::leaf(42)),
                        Err(resolver::ResolveError {
                            message: "!".into(),
                        }),
                    ]
                    .into_iter(),
                ))),
                _ => Err(resolver::ResolveError::unknown_field(field_name, self)),
            }
        }
    }

    let schema = Schema::parse_and_validate(sdl, "schema.graphql").unwrap();
    let document = ExecutableDocument::parse_and_validate(&schema, query, "query.graphql").unwrap();
    let operation = document.operations.get(None).unwrap();
    let variable_values = JsonMap::new();
    let variable_values =
        crate::request::coerce_variable_values(&schema, operation, &variable_values).unwrap();
    let root_operation_object_type_def = schema.get_object("Query").unwrap();
    let mut errors = Vec::new();
    let path = None;
    let initial_value = InitialValue;
    let implementers_map = OnceCell::new();
    let mut context = ExecutionContext {
        schema: &schema,
        document: &document,
        variable_values: &variable_values,
        errors: &mut errors,
        implementers_map: MaybeLazy::Lazy(&implementers_map),
    };
    let data = execute_selection_set(
        &mut context,
        path,
        ExecutionMode::Normal,
        root_operation_object_type_def,
        Some(&initial_value),
        &operation.selection_set.selections,
    )
    .ok();
    let response = crate::response::ExecutionResponse { data, errors };
    let response = serde_json::to_string_pretty(&response).unwrap();
    expect_test::expect![[r#"
        {
          "errors": [
            {
              "message": "resolver error: !",
              "locations": [
                {
                  "line": 1,
                  "column": 3
                }
              ],
              "path": [
                "f",
                1
              ]
            }
          ],
          "data": {
            "f": null
          }
        }"#]]
    .assert_eq(&response);
}
