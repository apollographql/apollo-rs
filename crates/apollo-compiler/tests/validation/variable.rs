use apollo_compiler::parse_mixed_validate;

#[test]
fn it_raises_undefined_variable_in_query_error() {
    let input = r#"
query ExampleQuery {
  topProducts(first: $undefinedVariable) {
    name
  }

  me {
    ... on User {
      id
      name
      profilePic(size: $dimensions)
      status
    }
  }
}

type Query {
  topProducts(first: Int): Products
  me: User
}

type User {
    id: ID
    name: String
    profilePic(size: Int): String
    status(membership: String): String
}

type Products {
  weight: Float
  size: Int
  name: String
}
"#;

    let errors = parse_mixed_validate(input, "schema.graphql")
        .unwrap_err()
        .to_string();
    assert!(
        errors.contains("variable `$undefinedVariable` is not defined"),
        "{errors}"
    );
    assert!(
        errors.contains("variable `$dimensions` is not defined"),
        "{errors}"
    );
}

#[test]
fn it_raises_unused_variable_in_query_error() {
    let input = r#"
query ExampleQuery($unusedVariable: Int) {
  topProducts {
    name
  }
  ... multipleSubscriptions
}

type Query {
  topProducts(first: Int): Product,
}

type Product {
  name: String
  price(setPrice: Int): Int
}
"#;

    let errors = parse_mixed_validate(input, "schema.graphql")
        .unwrap_err()
        .to_string();
    assert!(
        errors.contains("unused variable: `$unusedVariable`"),
        "{errors}"
    );
}

#[test]
fn it_raises_undefined_variable_in_query_in_fragments_error() {
    let input = r#"
query ExampleQuery {
  topProducts {
    name
  }

  me {
    ... on User {
      id
      name
      status(membership: $goldStatus)
    }
  }

  ... fragmentOne
}

fragment fragmentOne on Query {
    profilePic(size: $dimensions)
}

type Query {
  topProducts: Product
  profilePic(size: Int): String
  me: User
}

type User {
    id: ID
    name: String
    status(membership: String): String
}

type Product {
  name: String
  price(setPrice: Int): Int
}
"#;

    let errors = parse_mixed_validate(input, "schema.graphql")
        .unwrap_err()
        .to_string();
    assert!(
        errors.contains("variable `$goldStatus` is not defined"),
        "{errors}"
    );
    assert!(
        errors.contains("variable `$dimensions` is not defined"),
        "{errors}"
    );
}
