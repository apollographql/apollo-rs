mod field_merging;
mod interface;
mod object;
mod operation;
mod recursion;
mod types;
mod variable;

use apollo_compiler::ast;
use apollo_compiler::parser::LineColumn;
use apollo_compiler::ExecutableDocument;
use apollo_compiler::Schema;
use std::ops::Range;

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
            diag.line_column_range(),
            Some(Range {
                start: LineColumn { line: 2, column: 1 },
                end: LineColumn { line: 4, column: 2 }
            })
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
          "message": "`q` fragment cannot reference itself",
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
            diag.line_column_range(),
            Some(Range {
                start: LineColumn { line: 8, column: 1 },
                end: LineColumn {
                    line: 10,
                    column: 2
                }
            })
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
          "message": "fragment `A` must be used in an operation",
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
            diag.line_column_range(),
            Some(Range {
                start: LineColumn {
                    line: 2,
                    column: 13,
                },
                end: LineColumn {
                    line: 2,
                    column: 37,
                }
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
            diag.line_column_range(),
            Some(Range {
                start: LineColumn {
                    line: 3,
                    column: 22,
                },
                end: LineColumn {
                    line: 3,
                    column: 23,
                }
            })
        );
        json.assert_eq(&serde_json::to_string_pretty(&diag.to_json()).unwrap());
    });

    let doc = ast::Document::parse(r#"{ ...A }"#, "unknown_frag.graphql").unwrap();
    let json = expect_test::expect![[r#"
        {
          "message": "cannot find fragment `A` in this document",
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
            diag.line_column_range(),
            Some(Range {
                start: LineColumn { line: 1, column: 3 },
                end: LineColumn { line: 1, column: 7 },
            })
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

#[test]
fn json_location_with_multibyte() {
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
    # กรุงเทพมหานคร อมรรัตนโกสินทร์ มหินทรายุธยา มหาดิลกภพ นพรัตนราชธานีบูรีรมย์ อุดมราชนิเวศน์มหาสถาน อมรพิมานอวตารสถิต สักกะทัตติยวิษณุกรรมประสิทธิ์
    obj { ...q }
    # City of angels, great city of immortals, magnificent city of the nine gems, seat of the king, city of royal palaces, home of gods incarnate, erected by Vishvakarman at Indra's behest.
}
"#;

    let schema = Schema::parse_and_validate(input_type_system, "schema.graphql").unwrap();
    let err = ExecutableDocument::parse_and_validate(&schema, input_executable, "query.graphql")
        .expect_err("should have a validation error");

    let actual = err.to_string();
    let expected = expect_test::expect![[r#"
        Error: cannot find fragment `q` in this document
           ╭─[ query.graphql:4:11 ]
           │
         4 │     obj { ...q }
           │           ──┬─  
           │             ╰─── fragment `q` is not defined
        ───╯
    "#]];
    expected.assert_eq(&actual);

    let first_error = err.errors.iter().next().unwrap();
    let actual = serde_json::to_string_pretty(&first_error.to_json()).unwrap();
    let expected = expect_test::expect![[r#"
        {
          "message": "cannot find fragment `q` in this document",
          "locations": [
            {
              "line": 4,
              "column": 11
            }
          ]
        }"#]];
    expected.assert_eq(&actual);
}
