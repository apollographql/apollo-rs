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
        errors.contains("Org.organization is an object type and must select fields"),
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
