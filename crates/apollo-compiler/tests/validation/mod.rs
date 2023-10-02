mod interface;
mod object;
mod operation;
mod variable;

use apollo_compiler::validation::ValidationDatabase;
use apollo_compiler::ApolloCompiler;
use apollo_compiler::ReprDatabase;

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

    let mut compiler = ApolloCompiler::new();
    compiler.add_type_system(input_type_system, "schema.graphql");
    compiler.add_executable(input_executable, "query.graphql");

    let diagnostics = compiler.validate();
    assert!(diagnostics.is_empty());
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

    let mut compiler = ApolloCompiler::new();
    compiler.add_type_system(input_type_system, "schema.graphql");
    let id = compiler.add_executable(input_executable, "query.graphql");

    let schema = compiler.db.schema();
    let diagnostics = compiler
        .db
        .executable_document(id)
        .validate(&schema)
        .unwrap_err()
        .to_string();
    assert!(
        diagnostics.contains("an executable document must not contain an object type definition")
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

    let mut compiler = ApolloCompiler::new();
    compiler.add_type_system(input_type_system, "schema.graphql");
    compiler.add_executable(input_executable, "query.graphql");

    let diagnostics = compiler.validate();

    assert_eq!(diagnostics.len(), 1);
    assert_eq!(
        diagnostics[0].data.to_string(),
        "`q` fragment cannot reference itself"
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

    let mut compiler = ApolloCompiler::new();
    compiler.add_type_system(input_type_system, "schema.graphql");
    compiler.add_executable(input_executable, "query.graphql");

    let diagnostics = compiler.validate();

    assert_eq!(diagnostics.len(), 1);
    assert_eq!(
        diagnostics[0].data.to_string(),
        "`q` fragment cannot reference itself"
    );
}

#[test]
fn validation_with_precomputed_schema() {
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
        name
        nickname
    }
}
"#;

    let mut root_compiler = ApolloCompiler::new();
    root_compiler.add_type_system(input_type_system, "schema.graphql");
    assert!(root_compiler.validate().is_empty());

    let mut child_compiler = ApolloCompiler::from_schema(root_compiler.db.schema());
    let executable_id = child_compiler.add_executable(input_executable, "query.graphql");
    let diagnostics = child_compiler.db.validate_executable(executable_id);

    assert_eq!(diagnostics.len(), 1);
    assert_eq!(
        diagnostics[0].data.to_string(),
        "cannot query field `nickname` on type `TestObject`"
    );
}

#[test]
fn validation_without_type_system() {
    let mut compiler = ApolloCompiler::new();

    let valid_id = compiler.add_executable(r#"{ obj { name nickname } }"#, "valid.graphql");
    let diagnostics = compiler.db.validate_standalone_executable(valid_id);
    // We don't know what `obj` refers to, so assume it is valid.
    assert!(diagnostics.is_empty());

    let unused_frag_id = compiler.add_executable(
        r#"
            fragment A on Type { a }
            query { b }
        "#,
        "dupe_frag.graphql",
    );
    let diagnostics = compiler.db.validate_standalone_executable(unused_frag_id);
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(
        diagnostics[0].data.to_string(),
        "fragment `A` must be used in an operation"
    );

    let dupe_frag_id = compiler.add_executable(
        r#"
            fragment A on Type { a }
            fragment A on Type { b }
            query { ...A }
        "#,
        "dupe_frag.graphql",
    );
    let diagnostics = compiler.db.validate_standalone_executable(dupe_frag_id);
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(
        diagnostics[0].data.to_string(),
        "the fragment `A` is defined multiple times in the document"
    );

    let unknown_frag_id = compiler.add_executable(r#"{ ...A }"#, "unknown_frag.graphql");
    let diagnostics = compiler.db.validate_standalone_executable(unknown_frag_id);
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(
        diagnostics[0].data.to_string(),
        "cannot find fragment `A` in this document"
    );
}

#[test]
fn validate_variable_usage_without_type_system() {
    let mut compiler = ApolloCompiler::new();
    let id = compiler.add_executable(r#"
query nullableStringArg($nonNullableVar: String!, $nonNullableList: [String!]!, $nonNullableListList: [[Int!]!]) {
  arguments {
    nullableString(nullableString: $nonNullableVar)
    nullableList(nullableList: $nonNullableList)
    nullableListList(nullableListList: $nonNullableListList)
  }
}
"#, "query.graphql");

    let diagnostics = compiler.db.validate_standalone_executable(id);
    for diag in &diagnostics {
        println!("{diag}")
    }
    assert_eq!(diagnostics.len(), 0);
}
