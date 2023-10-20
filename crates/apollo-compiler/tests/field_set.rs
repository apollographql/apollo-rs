use apollo_compiler::executable::FieldSet;
use apollo_compiler::name;
use apollo_compiler::Schema;

fn common_schema() -> Schema {
    let input = r#"
        type Query {
            id: ID
            organization: Org
        }
        type Org {
            id: ID
        }
    "#;
    let schema = Schema::parse(input, "schema.graphql");
    schema.validate().unwrap();
    schema
}

#[test]
fn test_valid_field_sets() {
    let schema = common_schema();

    let input = "id";
    let field_set = FieldSet::parse(&schema, name!("Query"), input, "field_set.graphql");
    field_set.validate(&schema).unwrap();

    let input = "id organization { id }";
    let field_set = FieldSet::parse(&schema, name!("Query"), input, "field_set.graphql");
    field_set.validate(&schema).unwrap();
}

#[test]
fn test_invalid_field_sets() {
    let schema = common_schema();

    let input = "name";
    let field_set = FieldSet::parse(&schema, name!("Query"), input, "field_set.graphql");
    let errors = field_set
        .validate(&schema)
        .unwrap_err()
        .to_string_no_color();
    assert!(
        errors.contains("type `Query` does not have a field `name`"),
        "{errors}"
    );

    let input = "id organization";
    let field_set = FieldSet::parse(&schema, name!("Query"), input, "field_set.graphql");
    let errors = field_set
        .validate(&schema)
        .unwrap_err()
        .to_string_no_color();
    assert!(
        errors.contains("interface, union and object types must have a subselection set"),
        "{errors}"
    );
    assert!(
        errors.contains("field `organization` type `Org` is an object and must select fields"),
        "{errors}"
    );

    let input = "id(arg: true)";
    let field_set = FieldSet::parse(&schema, name!("Query"), input, "field_set.graphql");
    let errors = field_set
        .validate(&schema)
        .unwrap_err()
        .to_string_no_color();
    assert!(
        errors.contains("the argument `arg` is not supported"),
        "{errors}"
    );
}
