//! Directives applied to directive definitions, and the `DIRECTIVE_DEFINITION` location.
//! See <https://spec.graphql.org/draft/#DirectiveDefinition>

use apollo_compiler::ast::DirectiveLocation;
use apollo_compiler::introspection;
use apollo_compiler::request::coerce_variable_values;
use apollo_compiler::ExecutableDocument;
use apollo_compiler::Schema;
use expect_test::expect;

#[test]
fn parse_and_serialize() {
    let input = r#"
type Query {
  f: Int @old
}

directive @meta(note: String) on DIRECTIVE_DEFINITION

directive @old @deprecated(reason: "Use @new") @meta(note: "legacy") on FIELD_DEFINITION

directive @new(arg: Int) @meta repeatable on FIELD_DEFINITION
"#;
    let schema = Schema::parse_and_validate(input, "schema.graphql").unwrap();

    let old = &schema.directive_definitions["old"];
    let names: Vec<_> = old.directives.iter().map(|d| d.name.as_str()).collect();
    assert_eq!(names, ["deprecated", "meta"]);
    assert_eq!(
        schema.directive_definitions["meta"].locations,
        [DirectiveLocation::DirectiveDefinition]
    );
    assert!(schema.directive_definitions["deprecated"]
        .locations
        .contains(&DirectiveLocation::DirectiveDefinition));

    expect![[r#"
        directive @meta(note: String) on DIRECTIVE_DEFINITION

        directive @old @deprecated(reason: "Use @new") @meta(note: "legacy") on FIELD_DEFINITION

        directive @new(arg: Int) @meta repeatable on FIELD_DEFINITION

        type Query {
          f: Int @old
        }
    "#]]
    .assert_eq(&schema.to_string());

    // Serialized output parses back to the same schema
    let reparsed = Schema::parse_and_validate(schema.to_string(), "reparsed.graphql").unwrap();
    assert_eq!(reparsed.to_string(), schema.to_string());
}

#[test]
fn directive_in_invalid_location() {
    let input = r#"
type Query { f: Int }

directive @onField on FIELD_DEFINITION
directive @example @onField on QUERY
"#;
    let errors = Schema::parse_and_validate(input, "schema.graphql")
        .unwrap_err()
        .errors
        .to_string();
    assert!(
        errors.contains("onField directive is not supported for DIRECTIVE_DEFINITION location"),
        "{errors}"
    );
}

#[test]
fn undefined_directive() {
    let input = r#"
type Query { f: Int }

directive @example @undefined on QUERY
"#;
    let errors = Schema::parse_and_validate(input, "schema.graphql")
        .unwrap_err()
        .errors
        .to_string();
    assert!(
        errors.contains("cannot find directive `@undefined`"),
        "{errors}"
    );
}

#[test]
fn non_repeatable_directive_used_twice() {
    let input = r#"
type Query { f: Int }

directive @meta on DIRECTIVE_DEFINITION
directive @example @meta @meta on QUERY
"#;
    let errors = Schema::parse_and_validate(input, "schema.graphql")
        .unwrap_err()
        .errors
        .to_string();
    assert!(
        errors.contains("non-repeatable directive meta can only be used once per location"),
        "{errors}"
    );
}

#[test]
fn directly_self_referential() {
    let input = r#"
type Query { f: Int }

directive @selfRef @selfRef on DIRECTIVE_DEFINITION
"#;
    let errors = Schema::parse_and_validate(input, "schema.graphql")
        .unwrap_err()
        .errors
        .to_string();
    assert!(
        errors.contains("`selfRef` directive definition cannot reference itself"),
        "{errors}"
    );
}

#[test]
fn indirectly_self_referential() {
    // `@a` is applied to `@b`'s definition, and `@b` is applied to `@a`'s definition.
    let input = r#"
type Query { f: Int }

directive @a @b on DIRECTIVE_DEFINITION
directive @b @a on DIRECTIVE_DEFINITION
"#;
    let errors = Schema::parse_and_validate(input, "schema.graphql")
        .unwrap_err()
        .errors
        .to_string();
    assert!(
        errors.contains("`a` directive definition cannot reference itself"),
        "{errors}"
    );
    assert!(
        errors.contains("`b` directive definition cannot reference itself"),
        "{errors}"
    );

    // Through an argument type
    let input = r#"
type Query { f: Int }

directive @a(arg: In) on DIRECTIVE_DEFINITION | INPUT_FIELD_DEFINITION
directive @b @a on INPUT_FIELD_DEFINITION
input In { field: Int @b }
"#;
    let errors = Schema::parse_and_validate(input, "schema.graphql")
        .unwrap_err()
        .errors
        .to_string();
    assert!(
        errors.contains("`a` directive definition cannot reference itself"),
        "{errors}"
    );
}

#[test]
fn introspection_deprecated_directive() {
    let input = r#"
type Query { f: Int }

directive @old(arg: Int) @deprecated(reason: "Use @new") on FIELD_DEFINITION
directive @new(arg: Int) on FIELD_DEFINITION
"#;
    let schema = Schema::parse_and_validate(input, "schema.graphql").unwrap();
    let query = r#"
{
  __schema {
    directives {
      name
      isDeprecated
      deprecationReason
    }
  }
}
"#;
    let document = ExecutableDocument::parse_and_validate(&schema, query, "query.graphql").unwrap();
    let operation = document.operations.get(None).unwrap();
    let variables = coerce_variable_values(&schema, operation, &Default::default()).unwrap();
    let response = introspection::partial_execute(
        &schema,
        &schema.implementers_map(),
        &document,
        operation,
        &variables,
    )
    .unwrap();
    let directives = &response.data.unwrap()["__schema"]["directives"];
    let find = |name: &str| {
        directives
            .as_array()
            .unwrap()
            .iter()
            .find(|d| d["name"] == name)
            .map(|d| serde_json::to_value(d).unwrap())
            .unwrap()
    };
    assert_eq!(
        find("old"),
        serde_json::json!({
            "name": "old",
            "isDeprecated": true,
            "deprecationReason": "Use @new",
        })
    );
    assert_eq!(
        find("new"),
        serde_json::json!({
            "name": "new",
            "isDeprecated": false,
            "deprecationReason": null,
        })
    );
}
