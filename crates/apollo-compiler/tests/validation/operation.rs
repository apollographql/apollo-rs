use apollo_compiler::parse_mixed_validate;

#[test]
fn it_fails_validation_with_missing_ident() {
    let input = r#"
query {
  cat {
    name
  }
}

query getPet {
  cat {
    name
  }
}

query getOtherPet {
  cat {
    nickname
  }
}

type Query {
  cat: Cat
}

type Cat {
  name: String
  nickname: String
  meowVolume: Int
}
"#;

    let errors = parse_mixed_validate(input, "schema.graphql")
        .unwrap_err()
        .to_string_no_color();
    assert!(
        errors.contains(
            "anonymous operation cannot be selected when the document contains other operations"
        ),
        "{errors}"
    );
}

#[test]
fn it_fails_validation_with_duplicate_operation_names() {
    let input = r#"
query getName {
  cat {
    name
  }
}

query getName {
  cat {
    name
    nickname
  }
}

type Query {
  cat: Pet
}

union CatOrDog = Cat | Dog

interface Pet {
  name: String
  nickname: String
}

type Dog implements Pet {
  name: String
  nickname: String
  barkVolume: Int
}

type Cat implements Pet {
  name: String
  nickname: String
  meowVolume: Int
}
"#;

    let errors = parse_mixed_validate(input, "schema.graphql")
        .unwrap_err()
        .to_string_no_color();
    assert!(
        errors.contains("the operation `getName` is defined multiple times in the document"),
        "{errors}"
    );
}

#[test]
fn it_validates_unique_operation_names() {
    let input = r#"
query getCatName {
  cat {
    name
  }
}

query getPetNickname {
  cat {
    nickname
  }
}

type Query {
  cat: Pet
}

union CatOrDog = Cat | Dog

interface Pet {
  name: String
  nickname: String
}

type Dog implements Pet {
  name: String
  nickname: String
  barkVolume: Int
}

type Cat implements Pet {
  name: String
  nickname: String
  meowVolume: Int
}
"#;

    let (_schema, _executable) = parse_mixed_validate(input, "schema.graphql").unwrap();
}

#[test]
fn it_raises_an_error_for_illegal_operations() {
    let input = r#"
subscription sub {
  newMessage {
    body
    sender
  }
}

type Query {
  cat: Pet
}

union CatOrDog = Cat | Dog

interface Pet {
  name: String
}

type Dog implements Pet {
  name: String
  nickname: String
  barkVolume: Int
}

type Cat implements Pet {
  name: String
  nickname: String
  meowVolume: Int
}
"#;

    let errors = parse_mixed_validate(input, "schema.graphql")
        .unwrap_err()
        .to_string_no_color();
    assert!(
        errors
            .contains("`subscription` is not defined in the schema and is therefore not supported"),
        "{errors}"
    );
}

#[test]
fn it_validates_fields_in_operations() {
    let input = r#"
query getProduct {
  name
  noName
  topProducts {
    inStock
    price
  }
}

type Query {
  name: String
  topProducts: Product
}

type Product {
  inStock: Boolean
  name: String
}
"#;

    let errors = parse_mixed_validate(input, "schema.graphql")
        .unwrap_err()
        .to_string_no_color();
    assert!(
        errors.contains("`Query` does not have a field `noName`"),
        "{errors}"
    );
    assert!(
        errors.contains("type `Product` does not have a field `price`"),
        "{errors}"
    );
}
