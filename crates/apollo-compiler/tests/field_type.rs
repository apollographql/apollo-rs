use apollo_compiler::schema::FieldType;

#[test]
fn test_valid_field_type() {
    let input = "String!";
    let field_type = FieldType::parse(input, "field_type.graphql").expect("expected a field type");
    assert!(field_type.1.is_empty());

    let input = "[[[[[Int!]!]!]!]!]!";
    let field_type = FieldType::parse(input, "field_type.graphql").expect("expected a field type");
    assert!(field_type.1.is_empty());
}

#[test]
fn test_invalid_field_type() {
    let input = "[[String]";
    let field_type = FieldType::parse(input, "field_type.graphql").expect("expected a field type");
    let errors = field_type.1.to_string_no_color();
    assert!(
        errors.contains("Error: syntax error: expected R_BRACK, got EOF"),
        "{errors}"
    );

    let input = "[]";
    let field_type = FieldType::parse(input, "field_type.graphql");
    match field_type {
        Ok(_) => panic!("this input should have no type"),
        Err(diag) => {
            let errors = diag.to_string_no_color();
            assert!(errors.contains("expected item type"), "{errors}");
            assert!(errors.contains("expected R_BRACK, got EOF"), "{errors}");
        }
    }
}
