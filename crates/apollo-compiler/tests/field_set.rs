use apollo_compiler::executable::FieldSet;
use apollo_compiler::name;
use apollo_compiler::validation::Valid;
use apollo_compiler::Schema;

fn common_schema() -> Valid<Schema> {
    let input = r#"
        type Query {
            id: ID
            organization: Org
        }
        type Org {
            id: ID
        }
    "#;
    Schema::parse_and_validate(input, "schema.graphql").unwrap()
}

#[test]
fn test_valid_field_sets() {
    let schema = common_schema();

    let input = "";
    FieldSet::parse_and_validate(&schema, name!("Query"), input, "field_set.graphql").unwrap();

    let input = "id";
    FieldSet::parse_and_validate(&schema, name!("Query"), input, "field_set.graphql").unwrap();

    let input = "id organization { id }";
    FieldSet::parse_and_validate(&schema, name!("Query"), input, "field_set.graphql").unwrap();
}

#[test]
fn test_invalid_field_sets() {
    let schema = common_schema();

    let input = "name";
    let errors = FieldSet::parse_and_validate(&schema, name!("Query"), input, "field_set.graphql")
        .unwrap_err()
        .errors
        .to_string();
    assert!(
        errors.contains("type `Query` does not have a field `name`"),
        "{errors}"
    );

    let input = "id organization";
    let errors = FieldSet::parse_and_validate(&schema, name!("Query"), input, "field_set.graphql")
        .unwrap_err()
        .errors
        .to_string();
    assert!(
        errors.contains("interface, union and object types must have a subselection set"),
        "{errors}"
    );
    assert!(
        errors.contains("`Query.organization` is an object type `Org` and must select fields"),
        "{errors}"
    );

    let input = ":id";
    let errors = FieldSet::parse_and_validate(&schema, name!("Query"), input, "field_set.graphql")
        .unwrap_err()
        .errors
        .to_string();
    assert!(
        errors.contains("expected the end of the field set"),
        "{errors}"
    );

    let input = "id(arg: true)";
    let errors = FieldSet::parse_and_validate(&schema, name!("Query"), input, "field_set.graphql")
        .unwrap_err()
        .errors
        .to_string();
    assert!(
        errors.contains("the argument `arg` is not supported"),
        "{errors}"
    );
}

#[test]
fn test_field_set_with_leftover_tokens() {
    let schema = common_schema();

    let errors = |input| {
        FieldSet::parse_and_validate(&schema, name!("Query"), input, "field_set.graphql")
            .unwrap_err()
            .errors
            .to_string()
    };

    expect_test::expect![[r#"
        Error: syntax error: expected the end of the field set
           ╭─[ field_set.graphql:1:4 ]
           │
         1 │ id }
           │    ┬  
           │    ╰── expected the end of the field set
        ───╯
    "#]]
    .assert_eq(&errors("id }"));
    expect_test::expect![[r#"
        Error: syntax error: expected the end of the field set
           ╭─[ field_set.graphql:1:1 ]
           │
         1 │ :id
           │ ┬  
           │ ╰── expected the end of the field set
        ───╯
    "#]]
    .assert_eq(&errors(":id"));
}
