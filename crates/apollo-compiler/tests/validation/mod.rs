mod interface;
mod object;
mod operation;
mod variable;

use apollo_compiler::ast;
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

    let schema = Schema::parse(input_type_system, "schema.graphql");
    let executable = ExecutableDocument::parse(&schema, input_executable, "query.graphql");

    schema.validate().unwrap();
    executable.validate(&schema).unwrap();
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

    let schema = Schema::parse(input_type_system, "schema.graphql");
    let executable = ExecutableDocument::parse(&schema, input_executable, "query.graphql");

    schema.validate().unwrap();
    let errors = executable
        .validate(&schema)
        .unwrap_err()
        .to_string_no_color();
    assert!(
        errors.contains("an executable document must not contain an object type definition"),
        "{errors}"
    );
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

    let schema = Schema::parse(input_type_system, "schema.graphql");
    let executable = ExecutableDocument::parse(&schema, input_executable, "query.graphql");

    schema.validate().unwrap();
    let errors = executable
        .validate(&schema)
        .unwrap_err()
        .to_string_no_color();
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

    let schema = Schema::parse(input_type_system, "schema.graphql");
    let executable = ExecutableDocument::parse(&schema, input_executable, "query.graphql");

    schema.validate().unwrap();
    let errors = executable
        .validate(&schema)
        .unwrap_err()
        .to_string_no_color();
    assert!(
        errors.contains("`q` fragment cannot reference itself"),
        "{errors}"
    );
}

#[test]
fn validation_without_type_system() {
    let doc = ast::Document::parse(r#"{ obj { name nickname } }"#, "valid.graphql");
    // We don't know what `obj` refers to, so assume it is valid.
    doc.validate_standalone_executable().unwrap();

    let doc = ast::Document::parse(
        r#"
            fragment A on Type { a }
            query { b }
        "#,
        "dupe_frag.graphql",
    );
    let errors = doc
        .validate_standalone_executable()
        .unwrap_err()
        .to_string_no_color();
    assert!(
        errors.contains("fragment `A` must be used in an operation"),
        "{errors}"
    );

    let doc = ast::Document::parse(
        r#"
            fragment A on Type { a }
            fragment A on Type { b }
            query { ...A }
        "#,
        "dupe_frag.graphql",
    );
    let errors = doc
        .validate_standalone_executable()
        .unwrap_err()
        .to_string_no_color();
    assert!(
        errors.contains("the fragment `A` is defined multiple times in the document"),
        "{errors}"
    );

    let doc = ast::Document::parse(r#"{ ...A }"#, "unknown_frag.graphql");
    let errors = doc
        .validate_standalone_executable()
        .unwrap_err()
        .to_string_no_color();
    assert!(
        errors.contains("cannot find fragment `A` in this document"),
        "{errors}"
    );
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
    let doc = ast::Document::parse(input, "query.graphql");
    doc.validate_standalone_executable().unwrap()
}
