//! Tests for GraphQL spec conformance of `apollo_compiler::resolvers` execution,
//! written as reproducers for known deviations.
//!
//! Expected behaviors below were cross-checked against the graphql-js (v17)
//! reference implementation.

use apollo_compiler::name;
use apollo_compiler::request::coerce_variable_values;
use apollo_compiler::resolvers;
use apollo_compiler::resolvers::ResolvedValue;
use apollo_compiler::response::ExecutionResponse;
use apollo_compiler::response::JsonValue;
use apollo_compiler::response::ResponseDataPathSegment::Field;
use apollo_compiler::response::ResponseDataPathSegment::ListIndex;
use apollo_compiler::ExecutableDocument;
use apollo_compiler::Schema;
use serde_json_bytes::json;

const SDL: &str = r#"
    "Any JSON value, either a scalar or nested in lists or objects"
    scalar JSON

    input In {
        a: Int = 7
        b: Int
    }

    type Nested {
        inner: Int
    }

    type Query {
        echo(arg: In): JSON
        echoJson(arg: JSON): JSON
        id: ID
        nullableItems: [Int]
        nonNullItems: [Int!]
        bang: Int!
        ok: Int
        nest: Nested
        f(bar: Float!): Int
    }
"#;

struct Root;
struct Nested;

impl resolvers::ObjectValue for Root {
    fn type_name(&self) -> &str {
        "Query"
    }

    fn resolve_field<'a>(
        &'a self,
        info: &'a resolvers::ResolveInfo<'a>,
    ) -> Result<ResolvedValue<'a>, resolvers::ExecutionError> {
        match info.field_name() {
            // Echo the coerced `arg` argument back as a custom scalar leaf
            "echo" | "echoJson" => Ok(ResolvedValue::leaf(
                info.arguments()
                    .get("arg")
                    .cloned()
                    .unwrap_or(JsonValue::Null),
            )),
            "id" => Ok(ResolvedValue::leaf(42)),
            "nullableItems" => Ok(ResolvedValue::List(Box::new(
                [
                    Ok(ResolvedValue::leaf(1)),
                    Err(resolvers::ExecutionError {
                        message: "boom".into(),
                    }),
                    Ok(ResolvedValue::leaf(2)),
                ]
                .into_iter(),
            ))),
            "nonNullItems" => Ok(ResolvedValue::List(Box::new(
                [
                    Err(resolvers::ExecutionError {
                        message: "boom 0".into(),
                    }),
                    Err(resolvers::ExecutionError {
                        message: "boom 1".into(),
                    }),
                ]
                .into_iter(),
            ))),
            "bang" => Err(resolvers::ExecutionError {
                message: "boom bang".into(),
            }),
            "ok" => Ok(ResolvedValue::leaf(1)),
            "nest" => Ok(ResolvedValue::object(Nested)),
            _ => Err(self.unknown_field_error(info)),
        }
    }
}

impl resolvers::ObjectValue for Nested {
    fn type_name(&self) -> &str {
        "Nested"
    }

    fn resolve_field<'a>(
        &'a self,
        info: &'a resolvers::ResolveInfo<'a>,
    ) -> Result<ResolvedValue<'a>, resolvers::ExecutionError> {
        match info.field_name() {
            "inner" => Ok(ResolvedValue::leaf(3)),
            _ => Err(self.unknown_field_error(info)),
        }
    }
}

fn execute(query: &str, variables: JsonValue) -> ExecutionResponse {
    let schema = Schema::parse_and_validate(SDL, "schema.graphql").unwrap();
    let document = ExecutableDocument::parse_and_validate(&schema, query, "query.graphql").unwrap();
    resolvers::Execution::new(&schema, &document)
        .raw_variable_values(variables.as_object().unwrap())
        .execute_sync(&Root)
        .unwrap()
}

/// The `data` entry of the response, with `None` represented as JSON null
/// like in the serialized response.
fn data(response: &ExecutionResponse) -> JsonValue {
    response
        .data
        .clone()
        .map(JsonValue::Object)
        .unwrap_or(JsonValue::Null)
}

/// An input object field whose value is a variable with no runtime value behaves
/// as if the field were not provided at all: the field definition's default value
/// applies, and a field without a default is absent from the coerced map
/// (not an explicit null).
///
/// <https://spec.graphql.org/September2025/#sec-Input-Objects.Input-Coercion>
#[test]
fn input_object_field_with_unprovided_variable_uses_field_default() {
    let response = execute(
        "query($v: Int, $w: Int) { echo(arg: {a: $v, b: $w}) }",
        json!({}),
    );
    assert!(response.errors.is_empty(), "{:?}", response.errors);
    assert_eq!(data(&response), json!({"echo": {"a": 7}}));
}

/// ID results are serialized as strings, even when resolved from an integer.
///
/// <https://spec.graphql.org/September2025/#sec-ID.Result-Coercion>
#[test]
fn numeric_id_result_is_serialized_as_string() {
    let response = execute("{ id }", json!({}));
    assert!(response.errors.is_empty(), "{:?}", response.errors);
    assert_eq!(data(&response), json!({"id": "42"}));
}

/// A resolver error for a list item with a nullable item type is handled at the
/// item position: that item becomes null, the remaining items still complete,
/// and the list itself is preserved.
///
/// <https://spec.graphql.org/September2025/#sec-Handling-Execution-Errors>
#[test]
fn nullable_list_item_resolver_error_nullifies_only_that_item() {
    let response = execute("{ nullableItems }", json!({}));
    assert_eq!(data(&response), json!({"nullableItems": [1, null, 2]}));
    assert_eq!(response.errors.len(), 1);
    assert_eq!(response.errors[0].message, "resolver error: boom");
    assert_eq!(
        response.errors[0].path,
        vec![Field(name!("nullableItems")), ListIndex(1)]
    );
}

/// With a non-null item type the item error propagates to the list position.
/// Like in graphql-js, execution of the remaining items is canceled,
/// so only the first item error is reported.
#[test]
fn non_null_list_item_resolver_error_nullifies_list() {
    let response = execute("{ nonNullItems }", json!({}));
    assert_eq!(data(&response), json!({"nonNullItems": null}));
    assert_eq!(response.errors.len(), 1);
    assert_eq!(response.errors[0].message, "resolver error: boom 0");
    assert_eq!(
        response.errors[0].path,
        vec![Field(name!("nonNullItems")), ListIndex(0)]
    );
}

/// When an execution error propagates through a non-null field to the enclosing
/// selection set, execution of the remaining sibling fields is canceled
/// (their resolvers do not run and no additional errors are reported),
/// like in graphql-js. Response data is unaffected: the whole selection set
/// is discarded either way.
#[test]
fn non_null_field_error_cancels_sibling_execution() {
    let response = execute("{ bang ok }", json!({}));
    assert_eq!(response.data, None, "expected `data: null`");
    assert_eq!(response.errors.len(), 1);
    assert_eq!(response.errors[0].message, "resolver error: boom bang");
    assert_eq!(response.errors[0].path, vec![Field(name!("bang"))]);
}

/// Variables nested inside a custom scalar literal are replaced
/// with their runtime values.
///
/// Validation accepts such a document: “value must be coercible to type
/// (with the assumption that any variableUsage nested within value will
/// represent a runtime value valid for usage in its position)”,
/// so execution must substitute the runtime value instead of erroring.
///
/// <https://spec.graphql.org/September2025/#sec-Values-of-Correct-Type>
#[test]
fn custom_scalar_argument_substitutes_nested_variables() {
    let response = execute(
        "query($v: Int) { echoJson(arg: {a: $v, b: [1, $v]}) }",
        json!({"v": 1}),
    );
    assert!(response.errors.is_empty(), "{:?}", response.errors);
    assert_eq!(data(&response), json!({"echoJson": {"a": 1, "b": [1, 1]}}));
}

/// A variable without a runtime value nested inside a custom scalar literal
/// is dropped from object values and becomes null in list values,
/// like in graphql-js.
#[test]
fn custom_scalar_argument_with_missing_nested_variable() {
    let response = execute(
        "query($v: Int) { echoJson(arg: {a: $v, b: [1, $v]}) }",
        json!({}),
    );
    assert!(response.errors.is_empty(), "{:?}", response.errors);
    assert_eq!(data(&response), json!({"echoJson": {"b": [1, null]}}));
}

/// The `if` argument of `@include` is a non-null `Boolean!`, so a variable
/// whose runtime value is null raises a execution error for the selection set
/// being collected. At the operation root this makes the response `data` null.
///
/// A nullable variable is valid there per the variable-usage exception for
/// variables with a non-null default value:
/// <https://spec.graphql.org/September2025/#sec-All-Variable-Usages-Are-Allowed>
#[test]
fn include_if_null_variable_raises_field_error() {
    let response = execute(
        "query($c: Boolean = true) { ok @include(if: $c) }",
        json!({"c": null}),
    );
    assert_eq!(response.data, None, "expected `data: null`");
    assert_eq!(response.errors.len(), 1);
    let message = &response.errors[0].message;
    assert!(
        message.contains("argument if of directive @include"),
        "unexpected message: {message}"
    );
    assert!(response.errors[0].path.is_empty());
}

/// Same as [`include_if_null_variable_raises_field_error`] for `@skip`.
#[test]
fn skip_if_null_variable_raises_field_error() {
    let response = execute(
        "query($c: Boolean = false) { ok @skip(if: $c) }",
        json!({"c": null}),
    );
    assert_eq!(response.data, None, "expected `data: null`");
    assert_eq!(response.errors.len(), 1);
    let message = &response.errors[0].message;
    assert!(
        message.contains("argument if of directive @skip"),
        "unexpected message: {message}"
    );
}

/// In a nested selection set, the execution error for a null `if` value is handled
/// at the nearest enclosing nullable field, like other execution errors.
#[test]
fn nested_include_if_null_variable_is_handled_at_nearest_field() {
    let response = execute(
        "query($c: Boolean = true) { ok nest { inner @include(if: $c) } }",
        json!({"c": null}),
    );
    assert_eq!(data(&response), json!({"ok": 1, "nest": null}));
    assert_eq!(response.errors.len(), 1);
    assert_eq!(response.errors[0].path, vec![Field(name!("nest"))]);
}

/// Integer variable values for Float inputs are accepted up to and including
/// 2^53 − 1, the largest integer magnitude that is always exactly representable
/// as an IEEE 754 double. Only "a value outside the available precision"
/// must raise a request error.
///
/// <https://spec.graphql.org/September2025/#sec-Float.Input-Coercion>
#[test]
fn float_variable_accepts_max_safe_integer_boundary() {
    let schema = Schema::parse_and_validate(SDL, "schema.graphql").unwrap();
    let document = ExecutableDocument::parse_and_validate(
        &schema,
        "query($bar: Float!) { f(bar: $bar) }",
        "query.graphql",
    )
    .unwrap();
    let operation = document.operations.get(None).unwrap();

    let max_safe_int = (1_i64 << 53) - 1;
    let values = json!({ "bar": max_safe_int });
    let coerced = coerce_variable_values(&schema, operation, values.as_object().unwrap())
        .expect("2^53 - 1 is exactly representable as an IEEE 754 double");
    assert_eq!(coerced.get("bar"), Some(&json!(max_safe_int)));

    let values = json!({ "bar": i64::MAX });
    coerce_variable_values(&schema, operation, values.as_object().unwrap())
        .expect_err("i64::MAX is not exactly representable as an IEEE 754 double");
}
