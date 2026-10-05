use apollo_compiler::ast;
use apollo_compiler::ExecutableDocument;
use apollo_compiler::Schema;

const SCHEMA: &str = r#"
type Query {
  human: Human
  name: String
}

type Human {
  name: String
}
"#;

fn schema() -> apollo_compiler::validation::Valid<Schema> {
    Schema::parse_and_validate(SCHEMA, "schema.graphql").expect("schema is valid")
}

#[test]
fn empty_subselection_on_object_is_valid() {
    let schema = schema();
    ExecutableDocument::parse_and_validate(&schema, "{ human {} }", "query.graphql")
        .expect("empty subselection on an object field is valid");
}

#[test]
fn empty_operation_selection_set_is_valid() {
    let schema = schema();
    ExecutableDocument::parse_and_validate(&schema, "query Empty {}", "query.graphql")
        .expect("empty operation selection set is valid");
}

#[test]
fn empty_subselection_on_scalar_is_a_validation_error() {
    let schema = schema();
    let errors = ExecutableDocument::parse_and_validate(&schema, "{ name {} }", "query.graphql")
        .expect_err("subselection on a scalar is invalid")
        .errors
        .to_string();
    assert!(
        errors.contains("must not have subselections"),
        "unexpected errors: {errors}"
    );
}

#[test]
fn missing_subselection_on_object_is_still_an_error() {
    let schema = schema();
    let errors = ExecutableDocument::parse_and_validate(&schema, "{ human }", "query.graphql")
        .expect_err("object field without subselection is invalid")
        .errors
        .to_string();
    assert!(
        errors.contains("must have a subselection set"),
        "unexpected errors: {errors}"
    );
}

#[test]
fn ast_round_trips_empty_subselection() {
    let doc = ast::Document::parse("{ human {} }", "query.graphql").expect("parses");
    let printed = doc.to_string();
    assert!(printed.contains("human {}"), "printed: {printed}");
    let reparsed = ast::Document::parse(&printed, "query.graphql").expect("reparses");
    assert_eq!(doc, reparsed);
}

#[test]
fn ast_distinguishes_leaf_from_empty_subselection() {
    let leaf = ast::Document::parse("{ human }", "query.graphql").expect("parses");
    let empty = ast::Document::parse("{ human {} }", "query.graphql").expect("parses");
    assert_ne!(leaf, empty);
}

#[test]
fn executable_round_trips_empty_subselection() {
    let schema = schema();
    let doc = ExecutableDocument::parse_and_validate(&schema, "{ human {} }", "query.graphql")
        .expect("valid");
    let printed = doc.to_string();
    assert!(printed.contains("human {}"), "printed: {printed}");
    ExecutableDocument::parse_and_validate(&schema, &printed, "query.graphql")
        .expect("printed document is still valid");
}
