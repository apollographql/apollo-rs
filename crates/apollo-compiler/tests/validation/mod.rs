mod interface;
mod object;
mod operation;
mod recursion;
mod types;
mod variable;

use apollo_compiler::ast;
use apollo_compiler::execution::GraphQLLocation;
use apollo_compiler::ExecutableDocument;
use apollo_compiler::Schema;

#[test]
fn executable_and_type_system_definitions() {
    let input_type_system = r#"
type Query {
    name: String
}
"#;
    let input_executable = r#"
fragment q on Query { name }
query {
    ...q
}
"#;

    let schema = Schema::parse_and_validate(input_type_system, "schema.graphql").unwrap();
    ExecutableDocument::parse_and_validate(&schema, input_executable, "query.graphql").unwrap();
}

#[test]
fn executable_definition_does_not_contain_type_system_definitions() {
    let input_type_system = r#"
type Query {
    name: String
}
"#;
    let input_executable = r#"
type Object {
    notAllowed: Boolean!
}
fragment q on Query { name }
query {
    ...q
}
"#;

    let json = expect_test::expect![[r#"
{
  "message": "an executable document must not contain an object type definition",
  "locations": [
    {
      "line": 2,
      "column": 1
    }
  ]
}"#]];

    let schema = Schema::parse_and_validate(input_type_system, "schema.graphql").unwrap();
    let diagnostics =
        ExecutableDocument::parse_and_validate(&schema, input_executable, "query.graphql")
            .unwrap_err()
            .errors;
    let errors = diagnostics.to_string();
    assert!(
        errors.contains("an executable document must not contain an object type definition"),
        "{errors}"
    );

    diagnostics.iter().for_each(|diag| {
        assert_eq!(
            diag.get_line_column(),
            Some(GraphQLLocation { line: 2, column: 1 })
        );
        json.assert_eq(&serde_json::to_string_pretty(&diag.to_json()).unwrap());
    });
}

#[test]
fn executable_definition_with_cycles_do_not_overflow_stack() {
    let input_type_system = r#"
type Query {
    name: String
}
"#;

    let input_executable = r#"
{
    ...q
}
fragment q on Query {
    ...q
}
"#;

    let schema = Schema::parse_and_validate(input_type_system, "schema.graphql").unwrap();
    let errors = ExecutableDocument::parse_and_validate(&schema, input_executable, "query.graphql")
        .unwrap_err()
        .errors
        .to_string();
    assert!(
        errors.contains("`q` fragment cannot reference itself"),
        "{errors}"
    );
}

#[test]
fn executable_definition_with_nested_cycles_do_not_overflow_stack() {
    let input_type_system = r#"
type Query {
    obj: TestObject
}

type TestObject {
    name: String
}
"#;

    let input_executable = r#"
{
    obj {
        ...q
    }
}

fragment q on TestObject {
    ...q
}
"#;
    let json = expect_test::expect![[r#"
{
  "message": "compiler error: `q` fragment cannot reference itself",
  "locations": [
    {
      "line": 8,
      "column": 1
    }
  ]
}"#]];

    let schema = Schema::parse_and_validate(input_type_system, "schema.graphql").unwrap();
    let diagnostics =
        ExecutableDocument::parse_and_validate(&schema, input_executable, "query.graphql")
            .unwrap_err()
            .errors;
    let errors = diagnostics.to_string();
    assert!(
        errors.contains("`q` fragment cannot reference itself"),
        "{errors}"
    );
    diagnostics.iter().for_each(|diag| {
        assert_eq!(
            diag.get_line_column(),
            Some(GraphQLLocation { line: 8, column: 1 })
        );
        json.assert_eq(&serde_json::to_string_pretty(&diag.to_json()).unwrap());
    });
}

#[test]
fn validation_without_type_system() {
    let doc = ast::Document::parse(r#"{ obj { name nickname } }"#, "valid.graphql").unwrap();
    // We don't know what `obj` refers to, so assume it is valid.
    doc.validate_standalone_executable().unwrap();

    let doc = ast::Document::parse(
        r#"
            fragment A on Type { a }
            query { b }
        "#,
        "dupe_frag.graphql",
    )
    .unwrap();
    let json = expect_test::expect![[r#"
{
  "message": "compiler error: fragment `A` must be used in an operation",
  "locations": [
    {
      "line": 2,
      "column": 13
    }
  ]
}"#]];
    let diagnostics = doc.validate_standalone_executable().unwrap_err();
    let errors = diagnostics.to_string();
    assert!(
        errors.contains("fragment `A` must be used in an operation"),
        "{errors}"
    );
    diagnostics.iter().for_each(|diag| {
        assert_eq!(
            diag.get_line_column(),
            Some(GraphQLLocation {
                line: 2,
                column: 13
            })
        );
        json.assert_eq(&serde_json::to_string_pretty(&diag.to_json()).unwrap());
    });

    let doc = ast::Document::parse(
        r#"
            fragment A on Type { a }
            fragment A on Type { b }
            query { ...A }
        "#,
        "dupe_frag.graphql",
    )
    .unwrap();
    let json = expect_test::expect![[r#"
{
  "message": "the fragment `A` is defined multiple times in the document",
  "locations": [
    {
      "line": 3,
      "column": 22
    }
  ]
}"#]];
    let diagnostics = doc.validate_standalone_executable().unwrap_err();
    let errors = diagnostics.to_string();
    assert!(
        errors.contains("the fragment `A` is defined multiple times in the document"),
        "{errors}"
    );
    diagnostics.iter().for_each(|diag| {
        assert_eq!(
            diag.get_line_column(),
            Some(GraphQLLocation {
                line: 3,
                column: 22
            })
        );
        json.assert_eq(&serde_json::to_string_pretty(&diag.to_json()).unwrap());
    });

    let doc = ast::Document::parse(r#"{ ...A }"#, "unknown_frag.graphql").unwrap();
    let json = expect_test::expect![[r#"
{
  "message": "compiler error: cannot find fragment `A` in this document",
  "locations": [
    {
      "line": 1,
      "column": 3
    }
  ]
}"#]];
    let diagnostics = doc.validate_standalone_executable().unwrap_err();
    let errors = diagnostics.to_string();
    assert!(
        errors.contains("cannot find fragment `A` in this document"),
        "{errors}"
    );
    diagnostics.iter().for_each(|diag| {
        assert_eq!(
            diag.get_line_column(),
            Some(GraphQLLocation { line: 1, column: 3 })
        );
        json.assert_eq(&serde_json::to_string_pretty(&diag.to_json()).unwrap());
    });
}

#[test]
fn validate_variable_usage_without_type_system() {
    let input = r#"
    query nullableStringArg($nonNullableVar: String!, $nonNullableList: [String!]!, $nonNullableListList: [[Int!]!]) {
      arguments {
        nullableString(nullableString: $nonNullableVar)
        nullableList(nullableList: $nonNullableList)
        nullableListList(nullableListList: $nonNullableListList)
      }
    }
    "#;
    let doc = ast::Document::parse(input, "query.graphql").unwrap();
    doc.validate_standalone_executable().unwrap()
}
