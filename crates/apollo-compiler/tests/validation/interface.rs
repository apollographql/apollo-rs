use apollo_compiler::parse_mixed;
use apollo_compiler::Schema;

#[test]
fn it_fails_validation_with_duplicate_operation_fields() {
    let input = r#"
type Query implements NamedEntity {
  imgSize: Int
  name: String
  image: URL
  results: [Int]
}

interface NamedEntity {
  name: String
  image: URL
  results: [Int]
  name: String
}

scalar URL @specifiedBy(url: "https://tools.ietf.org/html/rfc3986")
"#;
    let schema = Schema::parse(input, "schema.graphql");
    let errors = schema
        .validate(Default::default())
        .unwrap_err()
        .to_string_no_color();
    assert!(
        errors
            .contains("duplicate definitions for the `name` field of interface type `NamedEntity`"),
        "{errors}"
    );
}

#[test]
fn it_fails_validation_with_duplicate_interface_definitions() {
    let input = r#"
type Query implements NamedEntity {
  imgSize: Int
  name: String
  image: URL
  results: [Int]
}

interface NamedEntity {
  name: String
  image: URL
  results: [Int]
}

interface NamedEntity {
  name: String
  image: URL
  results: [Int]
}

scalar URL @specifiedBy(url: "https://tools.ietf.org/html/rfc3986")
"#;
    let schema = Schema::parse(input, "schema.graphql");

    let errors = schema
        .validate(Default::default())
        .unwrap_err()
        .to_string_no_color();
    assert!(
        errors.contains("the type `NamedEntity` is defined multiple times"),
        "{errors}"
    );
}

#[test]
fn it_fails_validation_with_recursive_interface_definition() {
    let input = r#"
type Query implements NamedEntity {
  imgSize: Int
  name: String
  image: URL
  results: [Int]
}

interface NamedEntity implements NamedEntity {
  name: String
  image: URL
  results: [Int]
}

scalar URL @specifiedBy(url: "https://tools.ietf.org/html/rfc3986")
"#;
    let schema = Schema::parse(input, "schema.graphql");

    let errors = schema
        .validate(Default::default())
        .unwrap_err()
        .to_string_no_color();
    assert!(
        errors.contains("interface NamedEntity cannot implement itself"),
        "{errors}"
    );
}

#[test]
fn it_fails_validation_with_undefined_interface_definition() {
    let input = r#"
interface NamedEntity implements NewEntity {
  name: String
  image: URL
  results: [Int]
}

scalar URL @specifiedBy(url: "https://tools.ietf.org/html/rfc3986")
"#;
    let schema = Schema::parse(input, "schema.graphql");

    let errors = schema
        .validate(Default::default())
        .unwrap_err()
        .to_string_no_color();
    assert!(
        errors.contains("cannot find type `NewEntity` in this document"),
        "{errors}"
    );
}

#[test]
fn it_fails_validation_with_missing_transitive_interface() {
    let input = r#"
type Query implements Node {
  id: ID!
}

interface Node {
  id: ID!
}

interface Resource implements Node {
  id: ID!
  width: Int
}

interface Image implements Resource & Node {
  id: ID!
  thumbnail: String
}
"#;
    let schema = Schema::parse(input, "schema.graphql");

    let errors = schema
        .validate(Default::default())
        .unwrap_err()
        .to_string_no_color();
    assert!(
        errors.contains("type does not satisfy interface `Resource`: missing field `width`"),
        "{errors}"
    );
}

#[test]
fn it_generates_diagnostics_for_non_output_field_types() {
    let input = r#"
query mainPage {
  name {
    width
  }
}

type Query {
  name: mainInterface
}

interface mainInterface {
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
    let (schema, _executable) = parse_mixed(input, "doc.graphql");

    let errors = schema
        .validate(Default::default())
        .unwrap_err()
        .to_string_no_color();
    assert!(
        errors.contains("`coordinates` field must return an output type"),
        "{errors}"
    );
}
