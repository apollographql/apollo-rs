use crate::ast::Value;
use crate::collections::HashMap;
use crate::collections::HashSet;
use crate::collections::IndexMap;
use crate::executable::Field;
use crate::executable::Selection;
use crate::introspection::resolvers::SchemaMetaField;
use crate::parser::SourceMap;
use crate::parser::SourceSpan;
use crate::resolvers::input_coercion::coerce_argument_values;
use crate::resolvers::result_coercion::complete_value;
use crate::resolvers::MaybeAsync;
use crate::resolvers::MaybeAsyncObject;
use crate::resolvers::MaybeAsyncResolved;
use crate::resolvers::ResolveError;
use crate::resolvers::ResolveInfo;
use crate::resolvers::ResolvedValue;
use crate::response::GraphQLError;
use crate::response::JsonMap;
use crate::response::JsonValue;
use crate::response::ResponseDataPathSegment;
use crate::schema::ExtendedType;
use crate::schema::FieldDefinition;
use crate::schema::Implementers;
use crate::schema::ObjectType;
use crate::schema::Type;
use crate::validation::SuspectedValidationBug;
use crate::validation::Valid;
use crate::ExecutableDocument;
use crate::Name;
use crate::Schema;
use std::cell::OnceCell;

/// <https://spec.graphql.org/October2021/#sec-Normal-and-Serial-Execution>
#[derive(Debug, Copy, Clone)]
pub(crate) enum ExecutionMode {
    /// Allowed to resolve fields in any order, including in parallel
    Normal,
    /// Top-level fields of a mutation operation must be executed in order
    Sequential,
}

/// Return in `Err` when a field error occurred at some non-nullable place
///
/// <https://spec.graphql.org/October2021/#sec-Handling-Field-Errors>
pub(crate) struct PropagateNull;

/// Linked-list version of `Vec<PathElement>`, taking advantage of the call stack
pub(crate) type LinkedPath<'a> = Option<&'a LinkedPathElement<'a>>;

pub(crate) struct LinkedPathElement<'a> {
    pub(crate) element: ResponseDataPathSegment,
    pub(crate) next: LinkedPath<'a>,
}

pub(crate) struct ExecutionContext<'a> {
    pub(crate) schema: &'a Valid<Schema>,
    pub(crate) document: &'a Valid<ExecutableDocument>,
    pub(crate) variable_values: &'a Valid<JsonMap>,
    pub(crate) errors: &'a mut Vec<GraphQLError>,
    pub(crate) implementers_map: MaybeLazy<'a, HashMap<Name, Implementers>>,
    pub(crate) enable_schema_introspection: bool,
}

pub(crate) enum MaybeLazy<'a, T> {
    Eager(&'a T),
    Lazy(&'a OnceCell<T>),
}

impl<'a, T> Clone for MaybeLazy<'a, T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<'a, T> Copy for MaybeLazy<'a, T> {}

/// <https://spec.graphql.org/October2021/#ExecuteSelectionSet()>
///
/// `object_value: None` is a special case for top-level of `introspection::partial_execute`
pub(crate) async fn execute_selection_set<'a>(
    ctx: &mut ExecutionContext<'a>,
    path: LinkedPath<'_>,
    mode: ExecutionMode,
    object_type: &ObjectType,
    object_value: MaybeAsyncObject<'_>,
    selections: impl IntoIterator<Item = &'a Selection>,
) -> Result<JsonMap, PropagateNull> {
    let mut grouped_field_set = IndexMap::default();
    collect_fields(
        ctx,
        object_type,
        selections,
        &mut HashSet::default(),
        &mut grouped_field_set,
    );

    match mode {
        ExecutionMode::Normal => {
            // If we want parallelism, use `StreamExt::buffer_unordered` (async)
            // or Rayon’s `par_iter` (sync) here.
        }
        ExecutionMode::Sequential => {}
    }

    let mut response_map = JsonMap::with_capacity(grouped_field_set.len());
    for (&response_key, fields) in &grouped_field_set {
        // Indexing should not panic: `collect_fields` only creates a `Vec` to push to it
        let field_name = &fields[0].name;
        let Ok(field_def) = ctx.schema.type_field(&object_type.name, field_name) else {
            // TODO: Return a `validation_bug`` field error here?
            // The spec specifically has a “If fieldType is defined” condition,
            // but it being undefined would make the request invalid, right?
            continue;
        };
        let field_path = LinkedPathElement {
            element: ResponseDataPathSegment::Field(response_key.clone()),
            next: path,
        };
        if let Some(value) = execute_field(
            ctx,
            Some(&field_path),
            mode,
            object_type,
            object_value,
            field_def,
            fields,
        )
        .await?
        {
            response_map.insert(response_key.as_str(), value);
        }
    }
    Ok(response_map)
}

/// <https://spec.graphql.org/October2021/#CollectFields()>
fn collect_fields<'a>(
    ctx: &mut ExecutionContext<'a>,
    object_type: &ObjectType,
    selections: impl IntoIterator<Item = &'a Selection>,
    visited_fragments: &mut HashSet<&'a Name>,
    grouped_fields: &mut IndexMap<&'a Name, Vec<&'a Field>>,
) {
    for selection in selections {
        if eval_if_arg(selection, "skip", ctx.variable_values).unwrap_or(false)
            || !eval_if_arg(selection, "include", ctx.variable_values).unwrap_or(true)
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
                let Some(fragment) = ctx.document.fragments.get(&spread.fragment_name) else {
                    continue;
                };
                if !does_fragment_type_apply(ctx.schema, object_type, fragment.type_condition()) {
                    continue;
                }
                collect_fields(
                    ctx,
                    object_type,
                    &fragment.selection_set.selections,
                    visited_fragments,
                    grouped_fields,
                )
            }
            Selection::InlineFragment(inline) => {
                if let Some(condition) = &inline.type_condition {
                    if !does_fragment_type_apply(ctx.schema, object_type, condition) {
                        continue;
                    }
                }
                collect_fields(
                    ctx,
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
    object_type: &ObjectType,
    fragment_type: &Name,
) -> bool {
    match schema.types.get(fragment_type) {
        Some(ExtendedType::Object(_)) => *fragment_type == object_type.name,
        Some(ExtendedType::Interface(_)) => {
            object_type.implements_interfaces.contains(fragment_type)
        }
        Some(ExtendedType::Union(def)) => def.members.contains(&object_type.name),
        // Undefined or not an output type: validation should have caught this
        _ => false,
    }
}

fn eval_if_arg(
    selection: &Selection,
    directive_name: &str,
    variable_values: &Valid<JsonMap>,
) -> Option<bool> {
    match selection
        .directives()
        .get(directive_name)?
        .specified_argument_by_name("if")?
        .as_ref()
    {
        Value::Boolean(value) => Some(*value),
        Value::Variable(var) => variable_values.get(var.as_str())?.as_bool(),
        _ => None,
    }
}

/// <https://spec.graphql.org/October2021/#ExecuteField()>
///
/// `object_value: None` is a special case for top-level of `introspection::partial_execute`
///
/// Return `Ok(None)` for silently skipping that field.
async fn execute_field<'a>(
    ctx: &mut ExecutionContext<'a>,
    path: LinkedPath<'_>,
    mode: ExecutionMode,
    object_type: &ObjectType,
    object_value: MaybeAsyncObject<'_>,
    field_def: &FieldDefinition,
    fields: &[&'a Field],
) -> Result<Option<JsonValue>, PropagateNull> {
    let field = fields[0];
    let argument_values = match coerce_argument_values(ctx, path, field_def, field) {
        Ok(argument_values) => argument_values,
        Err(PropagateNull) if field_def.ty.is_non_null() => return Err(PropagateNull),
        Err(PropagateNull) => return Ok(Some(JsonValue::Null)),
    };
    let is_field_of_root_query = || {
        ctx.schema
            .schema_definition
            .query
            .as_ref()
            .is_some_and(|q| q.name == object_type.name)
    };
    let info = ResolveInfo {
        schema: ctx.schema,
        implementers_map: ctx.implementers_map,
        document: ctx.document,
        fields,
        arguments: &argument_values,
    };
    let resolved_result = match field.name.as_str() {
        "__typename" => Ok(MaybeAsync::Sync(ResolvedValue::leaf(
            object_type.name.as_str(),
        ))),
        "__schema" if is_field_of_root_query() => resolve_schema_meta_field(ctx),
        "__type" if is_field_of_root_query() => resolve_type_meta_field(ctx, &info),
        _ => match object_value {
            MaybeAsync::Async(obj) => obj.resolve_field(&info).await.map(MaybeAsync::Async),
            MaybeAsync::Sync(obj) => obj.resolve_field(&info).map(MaybeAsync::Sync),
        },
    };
    let completed_result = match resolved_result {
        Ok(resolved) => complete_value(ctx, path, mode, field.ty(), resolved, fields).await,
        Err(ResolveError { message }) => {
            ctx.errors.push(GraphQLError::field_error(
                format!("resolver error: {message}"),
                path,
                field.name.location(),
                &ctx.document.sources,
            ));
            Err(PropagateNull)
        }
    };
    try_nullify(&field_def.ty, completed_result)
}

fn resolve_schema_meta_field<'a>(
    ctx: &ExecutionContext<'a>,
) -> Result<MaybeAsyncResolved<'a>, ResolveError> {
    check_schema_introspection_enabled(ctx)?;
    Ok(MaybeAsync::Sync(ResolvedValue::object(SchemaMetaField)))
}

fn resolve_type_meta_field<'a>(
    ctx: &ExecutionContext<'a>,
    info: &ResolveInfo<'a>,
) -> Result<MaybeAsyncResolved<'a>, ResolveError> {
    check_schema_introspection_enabled(ctx)?;
    if let Some(name) = info.arguments().get("name").and_then(|v| v.as_str()) {
        Ok(MaybeAsync::Sync(crate::introspection::resolvers::type_def(
            info, name,
        )))
    } else {
        // This should never happen: `coerce_argument_values()` returns a map that conforms
        // to the `__type(name: String!): __Type` definition
        // Still, in case of a bug prefer returning an error than panicking
        Err(ResolveError {
            message: "expected string argument `name`".into(),
        })
    }
}

fn check_schema_introspection_enabled<'a>(ctx: &ExecutionContext<'a>) -> Result<(), ResolveError> {
    if ctx.enable_schema_introspection {
        Ok(())
    } else {
        // Disabled by default in the `apollo_compiler::resolvers::Excecution` builder,
        // use `.enable_schema_introspection(true)` to enable
        Err(ResolveError {
            message: "schema introspection is disabled".into(),
        })
    }
}

/// Try to insert a propagated null if possible, or keep propagating it.
///
/// <https://spec.graphql.org/October2021/#sec-Handling-Field-Errors>
pub(crate) fn try_nullify(
    ty: &Type,
    result: Result<Option<JsonValue>, PropagateNull>,
) -> Result<Option<JsonValue>, PropagateNull> {
    match result {
        Ok(json) => Ok(json),
        Err(PropagateNull) => {
            if ty.is_non_null() {
                Err(PropagateNull)
            } else {
                Ok(Some(JsonValue::Null))
            }
        }
    }
}

pub(crate) fn path_to_vec(mut link: LinkedPath<'_>) -> Vec<ResponseDataPathSegment> {
    let mut path = Vec::new();
    while let Some(node) = link {
        path.push(node.element.clone());
        link = node.next;
    }
    path.reverse();
    path
}

impl GraphQLError {
    pub(crate) fn field_error(
        message: impl Into<String>,
        path: LinkedPath<'_>,
        location: Option<SourceSpan>,
        sources: &SourceMap,
    ) -> Self {
        let mut err = Self::new(message, location, sources);
        err.path = path_to_vec(path);
        err
    }
}

impl SuspectedValidationBug {
    pub(crate) fn into_field_error(
        self,
        sources: &SourceMap,
        path: LinkedPath<'_>,
    ) -> GraphQLError {
        let Self { message, location } = self;
        let mut err = GraphQLError::field_error(message, path, location, sources);
        err.extensions
            .insert("APOLLO_SUSPECTED_VALIDATION_BUG", true.into());
        err
    }
}
