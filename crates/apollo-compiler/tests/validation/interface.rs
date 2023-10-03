use apollo_compiler::ApolloCompiler;
use apollo_compiler::ReprDatabase;

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
    let mut compiler = ApolloCompiler::new();
    compiler.add_document(input, "schema.graphql");
    let diagnostics = compiler.db.schema().validate().unwrap_err();
    let diagnostics = format!("{diagnostics:#}");
    assert!(
        diagnostics
            .contains("duplicate definitions for the `name` field of interface type `NamedEntity`"),
        "{diagnostics}"
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
    let mut compiler = ApolloCompiler::new();
    compiler.add_document(input, "schema.graphql");

    let diagnostics = compiler.db.schema().validate().unwrap_err();
    let diagnostics = format!("{diagnostics:#}");
    assert!(
        diagnostics.contains("the type `NamedEntity` is defined multiple times"),
        "{diagnostics}"
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
    let mut compiler = ApolloCompiler::new();
    compiler.add_document(input, "schema.graphql");

    let diagnostics = compiler.validate();
    for diagnostic in &diagnostics {
        println!("{diagnostic}")
    }
    assert_eq!(diagnostics.len(), 1);
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
    let mut compiler = ApolloCompiler::new();
    compiler.add_document(input, "schema.graphql");

    let diagnostics = compiler.validate();
    for diagnostic in &diagnostics {
        println!("{diagnostic}")
    }
    assert_eq!(diagnostics.len(), 1);
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
    let mut compiler = ApolloCompiler::new();
    compiler.add_document(input, "schema.graphql");

    let diagnostics = compiler.validate();
    for diagnostic in &diagnostics {
        println!("{diagnostic}")
    }
    assert_eq!(diagnostics.len(), 1);
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
    let mut compiler = ApolloCompiler::new();
    compiler.add_document(input, "schema.graphql");

    let diagnostics = compiler.validate();
    for diagnostic in &diagnostics {
        println!("{diagnostic}")
    }
    assert_eq!(diagnostics.len(), 3);
}
