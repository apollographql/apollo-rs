//! Validation tests for `@oneOf` input objects.
//!
//! Covers all rules in GraphQL spec §3.10.1 (OneOf Input Objects) and the corresponding
//! executable-document rules in §5.6.3 (Input Object Field Values) and the variable-usage
//! rule that treats a @oneOf field position as non-null.
//!
//! Test parity target: graphql-js `ValuesOfCorrectTypeRule-test.ts` (oneOf section)
//! and `type-system/definition-test.ts` (oneOf section).
//!
//! Spec reference: <https://spec.graphql.org/draft/#sec-OneOf-Input-Objects>
use apollo_compiler::introspection;
use apollo_compiler::request::coerce_variable_values;
use apollo_compiler::response::JsonMap;
use apollo_compiler::validation::Valid;
use apollo_compiler::ExecutableDocument;
use apollo_compiler::Schema;
use expect_test::expect;
use std::sync::OnceLock;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn schema_with_one_of() -> &'static Valid<Schema> {
    static SCHEMA: OnceLock<Valid<Schema>> = OnceLock::new();
    SCHEMA.get_or_init(|| {
        Schema::parse_and_validate(
            r#"
            type Query { oneOfField(arg: OneOfInput): String }
            input OneOfInput @oneOf {
                stringField: String
                intField: Int
            }
            "#,
            "schema.graphql",
        )
        .expect("schema should be valid")
    })
}

// ---------------------------------------------------------------------------
// Schema-level validation — spec §3.10.1 rule 5a: fields must be nullable
// ---------------------------------------------------------------------------

#[test]
fn valid_one_of_with_nullable_fields() {
    Schema::parse_and_validate(
        r#"
        type Query { f: String }
        input Foo @oneOf {
            a: String
            b: Int
        }
        "#,
        "schema.graphql",
    )
    .expect("valid @oneOf schema should compile");
}

#[test]
fn invalid_one_of_field_non_null() {
    let errors = Schema::parse_and_validate(
        r#"
        type Query { f: String }
        input Foo @oneOf {
            a: String!
            b: Int
        }
        "#,
        "schema.graphql",
    )
    .expect_err("non-null field in @oneOf should fail")
    .errors;

    let expected = expect![[r#"
        Error: `Foo.a` field of a @oneOf input object must be nullable
           ╭─[ schema.graphql:4:13 ]
           │
         4 │             a: String!
           │             ─────┬────  
           │                  ╰────── field `Foo.a` defined here
           │                  │      
           │                  ╰────── remove the `!` to make this field nullable
           │ 
           │ Help: Fields of a @oneOf input object must all be nullable and must not have default values.
        ───╯
    "#]];
    expected.assert_eq(&errors.to_string());
}

#[test]
fn invalid_one_of_multiple_non_null_fields() {
    let errors = Schema::parse_and_validate(
        r#"
        type Query { f: String }
        input Foo @oneOf {
            a: String!
            b: Int!
        }
        "#,
        "schema.graphql",
    )
    .expect_err("multiple non-null fields in @oneOf should fail")
    .errors
    .to_string();

    assert!(
        errors.contains("field of a @oneOf input object must be nullable"),
        "unexpected errors: {errors}"
    );
}

// ---------------------------------------------------------------------------
// Schema-level validation — spec §3.10.1 rule 5b: fields must not have defaults
// ---------------------------------------------------------------------------

/// Mirrors graphql-js: "rejects fields with default values"
#[test]
fn invalid_one_of_field_has_default_value() {
    let errors = Schema::parse_and_validate(
        r#"
        type Query { f: String }
        input Foo @oneOf {
            a: String = "hello"
            b: Int
        }
        "#,
        "schema.graphql",
    )
    .expect_err("default-value field in @oneOf should fail")
    .errors;

    let expected = expect![[r#"
        Error: `Foo.a` field of a @oneOf input object must not have a default value
           ╭─[ schema.graphql:4:13 ]
           │
         4 │             a: String = "hello"
           │             ─────────┬─────┬───  
           │                      ╰─────────── remove the default value
           │                            │     
           │                            ╰───── default value for `Foo.a` defined here
           │ 
           │ Help: Fields of a @oneOf input object must all be nullable and must not have default values.
        ───╯
    "#]];
    expected.assert_eq(&errors.to_string());
}

#[test]
fn invalid_one_of_non_null_field_with_default() {
    // Both rules (non-null AND default) fire independently.
    let errors = Schema::parse_and_validate(
        r#"
        type Query { f: String }
        input Foo @oneOf {
            a: String! = "bad"
            b: Int
        }
        "#,
        "schema.graphql",
    )
    .expect_err("non-null field with default in @oneOf should fail")
    .errors
    .to_string();

    assert!(
        errors.contains("field of a @oneOf input object must be nullable"),
        "should flag non-null: {errors}"
    );
    assert!(
        errors.contains("must not have a default value"),
        "should flag default: {errors}"
    );
}

// ---------------------------------------------------------------------------
// Value coercion — valid cases  (spec §5.6.3)
// ---------------------------------------------------------------------------

#[test]
fn valid_one_of_single_string_field() {
    let schema = schema_with_one_of();
    ExecutableDocument::parse_and_validate(
        &schema,
        r#"{ oneOfField(arg: { stringField: "hello" }) }"#,
        "query.graphql",
    )
    .expect("exactly one non-null field should be valid");
}

#[test]
fn valid_one_of_single_int_field() {
    let schema = schema_with_one_of();
    ExecutableDocument::parse_and_validate(
        &schema,
        r#"{ oneOfField(arg: { intField: 42 }) }"#,
        "query.graphql",
    )
    .expect("exactly one non-null field should be valid");
}

// ---------------------------------------------------------------------------
// Value coercion — invalid cases  (spec §5.6.3)
// ---------------------------------------------------------------------------

#[test]
fn invalid_one_of_no_fields() {
    let schema = schema_with_one_of();
    let errors = ExecutableDocument::parse_and_validate(
        &schema,
        r#"{ oneOfField(arg: {}) }"#,
        "query.graphql",
    )
    .expect_err("zero fields should fail")
    .errors;

    let expected = expect![[r#"
        Error: @oneOf input object `OneOfInput` must specify exactly one key, but 0 were given
           ╭─[ query.graphql:1:19 ]
           │
         1 │ { oneOfField(arg: {}) }
           │                   ─┬  
           │                    ╰── 0 fields were provided
           │ 
           │ Help: @oneOf input object `OneOfInput` requires exactly one non-null field.
        ───╯
    "#]];
    expected.assert_eq(&errors.to_string());
}

#[test]
fn invalid_one_of_multiple_fields() {
    let schema = schema_with_one_of();
    let errors = ExecutableDocument::parse_and_validate(
        &schema,
        r#"{ oneOfField(arg: { stringField: "a", intField: 1 }) }"#,
        "query.graphql",
    )
    .expect_err("two fields should fail")
    .errors;

    let expected = expect![[r#"
        Error: @oneOf input object `OneOfInput` must specify exactly one key, but 2 were given
           ╭─[ query.graphql:1:19 ]
           │
         1 │ { oneOfField(arg: { stringField: "a", intField: 1 }) }
           │                   ────────────────┬────────────────  
           │                                   ╰────────────────── 2 fields were provided
           │ 
           │ Help: @oneOf input object `OneOfInput` requires exactly one non-null field.
        ───╯
    "#]];
    expected.assert_eq(&errors.to_string());
}

#[test]
fn invalid_one_of_null_field() {
    let schema = schema_with_one_of();
    let errors = ExecutableDocument::parse_and_validate(
        &schema,
        r#"{ oneOfField(arg: { stringField: null }) }"#,
        "query.graphql",
    )
    .expect_err("null field should fail")
    .errors;

    let expected = expect![[r#"
        Error: `OneOfInput.stringField` value for @oneOf input object must be non-null
           ╭─[ query.graphql:1:34 ]
           │
         1 │ { oneOfField(arg: { stringField: null }) }
           │                                  ──┬─  
           │                                    ╰─── this value is null
           │ 
           │ Help: @oneOf input object `OneOfInput` field `stringField` must be non-null.
        ───╯
    "#]];
    expected.assert_eq(&errors.to_string());
}

// ---------------------------------------------------------------------------
// Variable usage in @oneOf fields
//
// Spec: a variable used as the sole value of a @oneOf field is in a
// "non-null position" regardless of the field's declared type — so the
// variable itself must be declared non-null.
//
// Mirrors graphql-js ValuesOfCorrectTypeRule-test.ts (oneOf section):
//   "Forbids one nullable variable"
//   "Allows exactly one non-nullable variable"
// ---------------------------------------------------------------------------

/// graphql-js: "Forbids one nullable variable"
#[test]
fn invalid_one_of_nullable_variable() {
    let schema = schema_with_one_of();
    let errors = ExecutableDocument::parse_and_validate(
        &schema,
        r#"query Q($var: String) { oneOfField(arg: { stringField: $var }) }"#,
        "query.graphql",
    )
    .expect_err("nullable variable in @oneOf field should fail")
    .errors;

    let expected = expect![[r#"
        Error: variable `$var` is of type `String` but must be non-nullable to be used for @oneOf input object `OneOfInput` field `stringField`
           ╭─[ query.graphql:1:56 ]
           │
         1 │ query Q($var: String) { oneOfField(arg: { stringField: $var }) }
           │                                                        ──┬─  
           │                                                          ╰─── variable `$var` has type `String`, which is nullable
           │ 
           │ Help: use `String!` to make this variable non-nullable for @oneOf input object `OneOfInput` field `stringField`.
        ───╯
    "#]];
    expected.assert_eq(&errors.to_string());
}

/// graphql-js: "Allows exactly one non-nullable variable"
#[test]
fn valid_one_of_non_null_variable() {
    let schema = schema_with_one_of();
    ExecutableDocument::parse_and_validate(
        &schema,
        r#"query Q($var: String!) { oneOfField(arg: { stringField: $var }) }"#,
        "query.graphql",
    )
    .expect("non-null variable in @oneOf field should be valid");
}

/// An undefined variable must NOT produce a @oneOf-specific error — the
/// existing UndefinedVariable rule already covers it.
#[test]
fn invalid_one_of_undefined_variable_no_oneof_error() {
    let schema = schema_with_one_of();
    let errors = ExecutableDocument::parse_and_validate(
        &schema,
        r#"{ oneOfField(arg: { stringField: $undeclared }) }"#,
        "query.graphql",
    )
    .expect_err("undefined variable should fail")
    .errors
    .to_string();

    // The UndefinedVariable rule fires.
    assert!(
        errors.contains("variable `$undeclared` is not defined"),
        "expected undefined-variable error, got: {errors}"
    );
    // The @oneOf nullable-variable rule must NOT fire for undefined vars.
    assert!(
        !errors.contains("must be non-nullable"),
        "@oneOf rule must not fire for undefined variable: {errors}"
    );
}

// ---------------------------------------------------------------------------
// Introspection — isOneOf field
//
// Spec §3.10.1 mandates that `__Type.isOneOf` returns true for OneOf Input
// Objects and false for all other types.
// ---------------------------------------------------------------------------

#[test]
fn introspection_is_one_of_true() {
    let schema = schema_with_one_of();

    let query = r#"
    {
        oneOfType: __type(name: "OneOfInput") { isOneOf }
        regularType: __type(name: "String") { isOneOf }
    }
    "#;

    let document = ExecutableDocument::parse_and_validate(&schema, query, "query.graphql")
        .expect("introspection query should be valid");

    let operation = document.operations.get(None).unwrap();
    let variables = coerce_variable_values(&schema, operation, &JsonMap::default()).unwrap();
    let response = introspection::partial_execute(
        &schema,
        &schema.implementers_map(),
        &document,
        operation,
        &variables,
    )
    .expect("introspection should succeed");

    // introspection::partial_execute wraps results in {"data": {...}}.
    let json = serde_json::to_value(&response).unwrap();
    let data = &json["data"];

    // OneOfInput has @oneOf → isOneOf must be true.
    assert_eq!(
        data["oneOfType"]["isOneOf"],
        serde_json::Value::Bool(true),
        "expected isOneOf=true for @oneOf type, got: {json}"
    );

    // String is not a @oneOf type → isOneOf must be false.
    assert_eq!(
        data["regularType"]["isOneOf"],
        serde_json::Value::Bool(false),
        "expected isOneOf=false for non-@oneOf type, got: {json}"
    );
}

/// A regular (non-@oneOf) input type must report isOneOf=false.
#[test]
fn introspection_is_one_of_false_for_regular_input() {
    let schema = Schema::parse_and_validate(
        r#"
        type Query { f(arg: RegularInput): String }
        input RegularInput { a: String b: Int }
        "#,
        "schema.graphql",
    )
    .expect("schema should be valid");

    let query = r#"{ __type(name: "RegularInput") { isOneOf } }"#;
    let document = ExecutableDocument::parse_and_validate(&schema, query, "query.graphql")
        .expect("introspection query should be valid");

    let operation = document.operations.get(None).unwrap();
    let variables = coerce_variable_values(&schema, operation, &JsonMap::default()).unwrap();
    let response = introspection::partial_execute(
        &schema,
        &schema.implementers_map(),
        &document,
        operation,
        &variables,
    )
    .expect("introspection should succeed");

    let json = serde_json::to_value(&response).unwrap();
    assert_eq!(
        json["data"]["__type"]["isOneOf"],
        serde_json::Value::Bool(false),
        "regular input object should have isOneOf=false, got: {json}"
    );
}

// ---------------------------------------------------------------------------
// Schema extensions — @oneOf must survive and be validated through extensions
// ---------------------------------------------------------------------------

#[test]
fn extending_oneof_type_with_nullable_field_is_valid() {
    // Adding a nullable field to a @oneOf type via extension is valid.
    Schema::parse_and_validate(
        r#"
        type Query { f: String }
        input Foo @oneOf { a: String }
        extend input Foo { b: Int }
        "#,
        "schema.graphql",
    )
    .expect("extending a @oneOf type with a nullable field should be valid");
}

#[test]
fn extending_oneof_type_with_nonnull_field_is_invalid() {
    // Adding a non-null field via extension must be rejected — the @oneOf
    // constraint on the base type applies to all fields regardless of where
    // they are introduced.
    let errors = Schema::parse_and_validate(
        r#"
        type Query { f: String }
        input Foo @oneOf { a: String }
        extend input Foo { b: Int! }
        "#,
        "schema.graphql",
    )
    .expect_err("non-null field added via extension should be invalid");
    let expected = expect![[r#"
        Error: `Foo.b` field of a @oneOf input object must be nullable
           ╭─[ schema.graphql:4:28 ]
           │
         4 │         extend input Foo { b: Int! }
           │                            ───┬───  
           │                               ╰───── field `Foo.b` defined here
           │                               │     
           │                               ╰───── remove the `!` to make this field nullable
           │ 
           │ Help: Fields of a @oneOf input object must all be nullable and must not have default values.
        ───╯
    "#]];
    expected.assert_eq(&errors.to_string());
}

#[test]
fn extending_oneof_type_with_default_value_is_invalid() {
    // Adding a field with a default value via extension must also be rejected.
    let errors = Schema::parse_and_validate(
        r#"
        type Query { f: String }
        input Foo @oneOf { a: String }
        extend input Foo { b: Int = 0 }
        "#,
        "schema.graphql",
    )
    .expect_err("field with default added via extension should be invalid");
    let expected = expect![[r#"
        Error: `Foo.b` field of a @oneOf input object must not have a default value
           ╭─[ schema.graphql:4:28 ]
           │
         4 │         extend input Foo { b: Int = 0 }
           │                            ─────┬───┬  
           │                                 ╰────── remove the default value
           │                                     │  
           │                                     ╰── default value for `Foo.b` defined here
           │ 
           │ Help: Fields of a @oneOf input object must all be nullable and must not have default values.
        ───╯
    "#]];
    expected.assert_eq(&errors.to_string());
}

#[test]
fn adding_oneof_via_extension_with_valid_base_type_is_valid() {
    // A regular input type whose fields are all nullable and have no defaults
    // can have @oneOf added via extension.
    Schema::parse_and_validate(
        r#"
        type Query { f: String }
        input Foo { a: String b: Int }
        extend input Foo @oneOf
        "#,
        "schema.graphql",
    )
    .expect("adding @oneOf via extension to a compatible input type should be valid");
}

#[test]
fn adding_oneof_via_extension_with_nonnull_field_in_base_is_invalid() {
    // If the base type already has a non-null field, applying @oneOf via
    // extension must be rejected because the merged type would violate the
    // @oneOf field-nullability rule.
    let errors = Schema::parse_and_validate(
        r#"
        type Query { f: String }
        input Foo { a: String! b: Int }
        extend input Foo @oneOf
        "#,
        "schema.graphql",
    )
    .expect_err("@oneOf extension on type with non-null field should be invalid");
    let expected = expect![[r#"
        Error: `Foo.a` field of a @oneOf input object must be nullable
           ╭─[ schema.graphql:3:21 ]
           │
         3 │         input Foo { a: String! b: Int }
           │                     ─────┬────  
           │                          ╰────── field `Foo.a` defined here
           │                          │      
           │                          ╰────── remove the `!` to make this field nullable
           │ 
           │ Help: Fields of a @oneOf input object must all be nullable and must not have default values.
        ───╯
    "#]];
    expected.assert_eq(&errors.to_string());
}
