use crate::executable::Field;
use crate::resolvers::execution::execute_selection_set;
use crate::resolvers::execution::try_nullify;
use crate::resolvers::execution::ExecutionContext;
use crate::resolvers::execution::ExecutionMode;
use crate::resolvers::execution::LinkedPath;
use crate::resolvers::execution::LinkedPathElement;
use crate::resolvers::execution::PropagateNull;
use crate::resolvers::AsyncObjectValue;
use crate::resolvers::AsyncResolvedValue;
use crate::resolvers::FieldError;
use crate::resolvers::MaybeAsync;
use crate::resolvers::MaybeAsyncResolved;
use crate::resolvers::ObjectValue;
use crate::resolvers::ResolvedValue;
use crate::response::GraphQLError;
use crate::response::JsonValue;
use crate::response::ResponseDataPathSegment;
use crate::schema::ExtendedType;
use crate::schema::Type;
use crate::validation::SuspectedValidationBug;
use futures::Stream;
use futures::StreamExt as _;
use std::pin::pin;
use std::pin::Pin;

enum LeafOrObject<'a> {
    Leaf(JsonValue),
    Object(MaybeAsync<Box<dyn AsyncObjectValue + 'a>, Box<dyn ObjectValue + 'a>>),
}

/// <https://spec.graphql.org/October2021/#CompleteValue()>
///
/// Returns `Err` for a field error being propagated upwards to find a nullable place
pub(crate) async fn complete_value<'a>(
    ctx: &mut ExecutionContext<'a>,
    path: LinkedPath<'_>,
    mode: ExecutionMode,
    ty: &'a Type,
    resolved: MaybeAsyncResolved<'_>,
    fields: &[&'a Field],
) -> Result<Option<JsonValue>, PropagateNull> {
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
    let resolved = match resolved {
        MaybeAsync::Async(AsyncResolvedValue::SkipForPartialExecution)
        | MaybeAsync::Sync(ResolvedValue::SkipForPartialExecution) => return Ok(None),

        MaybeAsync::Async(AsyncResolvedValue::Leaf(JsonValue::Null))
        | MaybeAsync::Sync(ResolvedValue::Leaf(JsonValue::Null)) => {
            if ty.is_non_null() {
                field_error!("non-null type {ty} resolved to null")
            } else {
                return Ok(Some(JsonValue::Null));
            }
        }
        MaybeAsync::Async(AsyncResolvedValue::List(stream)) => {
            let stream = pin!(stream.map(|result| result.map(MaybeAsync::Async)));
            return Box::pin(complete_list_value(ctx, path, mode, ty, fields, stream)).await;
        }
        MaybeAsync::Sync(ResolvedValue::List(iter)) => {
            let stream = futures::stream::iter(iter);
            let stream = pin!(stream.map(|result| result.map(MaybeAsync::Sync)));
            return Box::pin(complete_list_value(ctx, path, mode, ty, fields, stream)).await;
        }
        MaybeAsync::Async(AsyncResolvedValue::Leaf(leaf))
        | MaybeAsync::Sync(ResolvedValue::Leaf(leaf)) => LeafOrObject::Leaf(leaf),
        MaybeAsync::Async(AsyncResolvedValue::Object(obj)) => {
            LeafOrObject::Object(MaybeAsync::Async(obj))
        }
        MaybeAsync::Sync(ResolvedValue::Object(obj)) => LeafOrObject::Object(MaybeAsync::Sync(obj)),
    };

    let ty_name = match ty {
        Type::List(_) | Type::NonNullList(_) => {
            field_error!("list type {ty} resolved to an object")
        }
        Type::Named(name) | Type::NonNullNamed(name) => name,
    };
    let Some(ty_def) = ctx.schema.types.get(ty_name) else {
        ctx.errors.push(
            SuspectedValidationBug {
                message: format!("undefined type {ty_name}"),
                location,
            }
            .into_field_error(&ctx.document.sources, path),
        );
        return Err(PropagateNull);
    };
    if let ExtendedType::InputObject(_) = ty_def {
        ctx.errors.push(
            SuspectedValidationBug {
                message: format!("field with input object type {ty_name}"),
                location,
            }
            .into_field_error(&ctx.document.sources, path),
        );
        return Err(PropagateNull);
    }
    let resolved_obj = match resolved {
        LeafOrObject::Leaf(json_value) => {
            return complete_leaf_value(ctx, path, ty_name, ty_def, json_value, fields);
        }
        LeafOrObject::Object(resolved_obj) => resolved_obj,
    };
    let resolved_type_name = resolved_obj.type_name();
    let object_type = match ty_def {
        ExtendedType::InputObject(_) => unreachable!(), // early return above
        ExtendedType::Enum(_) | ExtendedType::Scalar(_) => {
            field_error!(
                "resolver returned a an object of type {resolved_type_name}, expected {ty_name}",
            )
        }
        ExtendedType::Interface(_) | ExtendedType::Union(_) => {
            let Some(object_def) = ctx.schema.get_object(resolved_type_name) else {
                field_error!(
                    "resolver returned an object of type {resolved_type_name} \
                     not defined in the schema"
                )
            };
            if let ExtendedType::Union(union_def) = ty_def {
                if !union_def.members.contains(resolved_type_name) {
                    field_error!(
                        "resolver returned an object of type {resolved_type_name}, \
                         expected a member of union type {ty_name}"
                    )
                }
            } else if !object_def.implements_interfaces.contains(ty_name) {
                field_error!(
                    "resolver returned an object of type {resolved_type_name} \
                     which does not implement interface {ty_name}"
                )
            }
            object_def
        }
        ExtendedType::Object(def) => {
            if resolved_type_name == ty_name.as_str() {
                def
            } else {
                field_error!(
                    "resolver returned an object of type {resolved_type_name}, expected {ty_name}"
                )
            }
        }
    };
    let resolved_obj = match &resolved_obj {
        MaybeAsync::Async(obj) => MaybeAsync::Async(&**obj),
        MaybeAsync::Sync(obj) => MaybeAsync::Sync(&**obj),
    };
    Box::pin(execute_selection_set(
        ctx,
        path,
        mode,
        object_type,
        resolved_obj,
        fields
            .iter()
            .flat_map(|field| &field.selection_set.selections),
    ))
    .await
    .map(|map| Some(JsonValue::Object(map)))
}

async fn complete_list_value<'a, 'b>(
    ctx: &mut ExecutionContext<'a>,
    path: LinkedPath<'_>,
    mode: ExecutionMode,
    ty: &'a Type,
    fields: &[&'a Field],
    stream: Pin<&mut dyn Stream<Item = Result<MaybeAsyncResolved<'b>, FieldError>>>,
) -> Result<Option<JsonValue>, PropagateNull> {
    let inner_ty = match ty {
        Type::Named(_) | Type::NonNullNamed(_) => {
            let location = fields[0].name.location();
            ctx.errors.push(GraphQLError::field_error(
                format!("Non-list type {ty} resolved to a list"),
                path,
                location,
                &ctx.document.sources,
            ));
            return Err(PropagateNull);
        }
        Type::List(inner_ty) | Type::NonNullList(inner_ty) => inner_ty,
    };
    let mut completed_list = Vec::with_capacity(stream.size_hint().0);
    let mut stream = stream.enumerate();
    while let Some((index, inner_result)) = stream.next().await {
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
        )
        .await;
        // On field error, try to nullify that item
        match try_nullify(inner_ty, inner_result) {
            Ok(None) => {}
            Ok(Some(inner_value)) => completed_list.push(inner_value),
            // If the item is non-null, try to nullify the list
            Err(PropagateNull) => return try_nullify(ty, Err(PropagateNull)),
        }
    }
    Ok(Some(completed_list.into()))
}

fn complete_leaf_value(
    ctx: &mut ExecutionContext<'_>,
    path: LinkedPath<'_>,
    ty_name: &crate::Name,
    ty_def: &ExtendedType,
    json_value: JsonValue,
    fields: &[&Field],
) -> Result<Option<JsonValue>, PropagateNull> {
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
    match ty_def {
        ExtendedType::InputObject(_) => unreachable!(), // early return above
        ExtendedType::Object(_) | ExtendedType::Interface(_) | ExtendedType::Union(_) => {
            field_error!("resolver returned a leaf value but expected an object for type {ty_name}")
        }
        ExtendedType::Enum(enum_def) => {
            // https://spec.graphql.org/October2021/#sec-Enums.Result-Coercion
            if !json_value
                .as_str()
                .is_some_and(|str| enum_def.values.contains_key(str))
            {
                field_error!("resolver returned {json_value}, expected enum {ty_name}")
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
                        field_error!("resolver returned {json_value} which overflows Int")
                    }
                } else {
                    field_error!("resolver returned {json_value}, expected Int")
                }
            }
            "Float" => {
                // https://spec.graphql.org/October2021/#sec-Float.Result-Coercion
                if !json_value.is_f64() {
                    field_error!("resolver returned {json_value}, expected Float")
                }
            }
            "String" => {
                // https://spec.graphql.org/October2021/#sec-String.Result-Coercion
                if !json_value.is_string() {
                    field_error!("resolver returned {json_value}, expected String")
                }
            }
            "Boolean" => {
                // https://spec.graphql.org/October2021/#sec-Boolean.Result-Coercion
                if !json_value.is_boolean() {
                    field_error!("resolver returned {json_value}, expected Boolean")
                }
            }
            "ID" => {
                // https://spec.graphql.org/October2021/#sec-ID.Result-Coercion
                if !(json_value.is_string() || json_value.is_i64()) {
                    field_error!("resolver returned {json_value}, expected ID")
                }
            }
            _ => {
                // Custom scalar: accept any JSON value (including an array or object,
                // despite this being a "leaf" as far as GraphQL resolution is concerned)
            }
        },
    };
    Ok(Some(json_value))
}

#[test]
fn test_error_path() {
    use crate::resolvers;
    use crate::ExecutableDocument;
    use crate::Schema;

    let sdl = "type Query { f: [Int] }";
    let query = "{ f }";

    struct InitialValue;

    impl resolvers::ObjectValue for InitialValue {
        fn type_name(&self) -> &str {
            "Query"
        }

        fn resolve_field<'a>(
            &'a self,
            info: &resolvers::ResolveInfo<'a>,
        ) -> Result<ResolvedValue<'a>, resolvers::FieldError> {
            match info.field_name() {
                "f" => Ok(ResolvedValue::List(Box::new(
                    [
                        Ok(ResolvedValue::leaf(42)),
                        Err(resolvers::FieldError {
                            message: "!".into(),
                        }),
                    ]
                    .into_iter(),
                ))),
                _ => Err(self.unknown_field_error(info)),
            }
        }
    }

    let schema = Schema::parse_and_validate(sdl, "schema.graphql").unwrap();
    let document = ExecutableDocument::parse_and_validate(&schema, query, "query.graphql").unwrap();
    let response = resolvers::Execution::new(&schema, &document)
        .execute_sync(&InitialValue)
        .unwrap();
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
