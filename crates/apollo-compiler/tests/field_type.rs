use apollo_compiler::schema::Type;

#[test]
fn test_valid_field_type() {
    let input = "String!";
    let field_type = Type::parse(input, "field_type.graphql").expect("expected a field type");
    assert_eq!(field_type.to_string(), input);

    let input = "[[[[[Int!]!]!]!]!]!";
    let field_type = Type::parse(input, "field_type.graphql").expect("expected a field type");
    assert_eq!(field_type.to_string(), input);
}

#[test]
fn test_invalid_field_type() {
    let input = "[[String]";
    match Type::parse(input, "field_type.graphql") {
        Ok(parsed) => panic!("Field type should fail to parse, instead got `{parsed}`"),
        Err(errors) => {
            let errors = errors.to_string_no_color();
            assert!(
                errors.contains("Error: syntax error: expected R_BRACK, got EOF"),
                "{errors}"
            );
        }
    }

    let input = "[]";
    match Type::parse(input, "field_type.graphql") {
        Ok(parsed) => panic!("Field type should fail to parse, instead got `{parsed}`"),
        Err(diag) => {
            let errors = diag.to_string_no_color();
            assert!(errors.contains("expected item type"), "{errors}");
            assert!(errors.contains("expected R_BRACK, got EOF"), "{errors}");
        }
    }
}
