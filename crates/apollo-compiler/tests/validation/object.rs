use apollo_compiler::parse_mixed_validate;

#[test]
fn it_generates_diagnostics_for_non_output_field_types() {
    let input = r#"
query mainPage {
  width
  result {
    ... on Person {
      name
    }
  }
  entity {
    name
  }
}

type Query {
  width: Int
  img: Url
  relationship: Person
  entity: NamedEntity
  depth: Number
  result: SearchResult
  permissions: Auth
  coordinates: Point2D
  main: mainPage
}

type Person {
  name: String
  age: Int
}

type Photo {
  size: Int
  type: String
}

interface NamedEntity {
  name: String
}

enum Number {
  INT
  FLOAT
}

union SearchResult = Photo | Person

directive @Auth(username: String!) repeatable on OBJECT | INTERFACE

input Point2D {
  x: Float
  y: Float
}

scalar Url @specifiedBy(url: "https://tools.ietf.org/html/rfc3986")
"#;
    let errors = parse_mixed_validate(input, "doc.graphql")
        .unwrap_err()
        .to_string();
    assert!(
        errors.contains("`coordinates` field must return an output type"),
        "{errors}"
    );
}
