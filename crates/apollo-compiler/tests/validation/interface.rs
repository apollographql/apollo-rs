use apollo_compiler::parser::Parser;
use apollo_compiler::Schema;
use expect_test::expect;

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
    let errors = Schema::parse_and_validate(input, "schema.graphql")
        .unwrap_err()
        .errors
        .to_string();
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
    let errors = Schema::parse_and_validate(input, "schema.graphql")
        .unwrap_err()
        .errors
        .to_string();
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
    let errors = Schema::parse_and_validate(input, "schema.graphql")
        .unwrap_err()
        .errors
        .to_string();
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
    let errors = Schema::parse_and_validate(input, "schema.graphql")
        .unwrap_err()
        .errors
        .to_string();
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
    let errors = Schema::parse_and_validate(input, "schema.graphql")
        .unwrap_err()
        .errors
        .to_string();
    assert!(
        errors
            .contains("type `Image` does not satisfy interface `Resource`: missing field `width`"),
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
    let errors = Parser::new()
        .parse_mixed_validate(input, "doc.graphql")
        .unwrap_err()
        .to_string();
    assert!(
        errors.contains("`coordinates` field must return an output type"),
        "{errors}"
    );
}

#[test]
fn it_fails_validation_when_object_field_type_is_not_subtype_of_interface_field() {
    let input = r#"
type Query implements Node {
  id: ID!
}

interface Node {
  id: ID!
}

interface Product {
  details: ProductDetails!
}

type ProductDetails {
  name: String
}

type DigitalProduct implements Product {
  details: ProductDetails
}
"#;
    let errors = Schema::parse_and_validate(input, "schema.graphql")
        .unwrap_err()
        .errors
        .to_string();
    let expected = expect![[r#"
        Error: interface field `Product.details` expects type `ProductDetails!` but `DigitalProduct.details` of type `ProductDetails` is not a proper subtype
            ╭─[ schema.graphql:19:3 ]
            │
         19 │   details: ProductDetails
            │   ───────────┬───────────  
            │              ╰───────────── field type is not a proper subtype of `Product.details`
            │
            ├─[ schema.graphql:19:3 ]
            │
         11 │   details: ProductDetails!
            │   ────────────┬───────────  
            │               ╰───────────── `Product.details` originally defined here
        ────╯
    "#]];
    expected.assert_eq(&errors);
}

#[test]
fn it_accepts_valid_covariant_interface_field_types() {
    let input = r#"
type Query implements Node {
  id: ID!
}

interface Node {
  id: ID!
}

interface Animal {
  name: String
}

type Dog implements Animal {
  name: String!
}
"#;
    Schema::parse_and_validate(input, "schema.graphql")
        .expect("Expected validation to succeed for covariant non-null field");
}

#[test]
fn it_fails_validation_when_list_item_type_is_not_subtype() {
    let input = r#"
type Query implements Node {
  id: ID!
}

interface Node {
  id: ID!
}

interface Collection {
  items: [Item!]!
}

type Item {
  name: String
}

type MyCollection implements Collection {
  items: [Item]!
}
"#;
    let errors = Schema::parse_and_validate(input, "schema.graphql")
        .unwrap_err()
        .errors
        .to_string();
    let expected = expect![[r#"
        Error: interface field `Collection.items` expects type `[Item!]!` but `MyCollection.items` of type `[Item]!` is not a proper subtype
            ╭─[ schema.graphql:19:3 ]
            │
         19 │   items: [Item]!
            │   ───────┬──────  
            │          ╰──────── field type is not a proper subtype of `Collection.items`
            │
            ├─[ schema.graphql:19:3 ]
            │
         11 │   items: [Item!]!
            │   ───────┬───────  
            │          ╰───────── `Collection.items` originally defined here
        ────╯
    "#]];
    expected.assert_eq(&errors);
}

#[test]
fn it_accepts_union_member_as_subtype_of_union_interface_field() {
    let input = r#"
type Query {
  search: SearchResult
}

union SearchResult = Photo | Person

type Photo {
  url: String
}

type Person {
  name: String
}

interface HasSearch {
  search: SearchResult
}

type SearchPage implements HasSearch {
  search: Photo
}
"#;
    Schema::parse_and_validate(input, "schema.graphql")
        .expect("Expected validation to succeed: union member is a valid subtype of union");
}

#[test]
fn it_accepts_interface_to_interface_covariance() {
    let input = r#"
type Query {
  node: Node
}

interface Node {
  id: ID!
}

interface Resource implements Node {
  id: ID!
  url: String
}

interface HasNode {
  node: Node
}

type ResourcePage implements HasNode {
  node: Resource
}
"#;
    Schema::parse_and_validate(input, "schema.graphql").expect(
        "Expected validation to succeed: interface implementing interface is a valid subtype",
    );
}

#[test]
fn it_fails_validation_when_interface_implementing_interface_has_non_covariant_field() {
    let input = r#"
type Query {
  node: Node
}

interface Node {
  id: ID!
}

interface Base {
  node: Node!
}

interface Child implements Base {
  node: Node
}
"#;
    let errors = Schema::parse_and_validate(input, "schema.graphql")
        .unwrap_err()
        .errors
        .to_string();
    let expected = expect![[r#"
        Error: interface field `Base.node` expects type `Node!` but `Child.node` of type `Node` is not a proper subtype
            ╭─[ schema.graphql:15:3 ]
            │
         15 │   node: Node
            │   ─────┬────  
            │        ╰────── field type is not a proper subtype of `Base.node`
            │
            ├─[ schema.graphql:15:3 ]
            │
         11 │   node: Node!
            │   ─────┬─────  
            │        ╰─────── `Base.node` originally defined here
        ────╯
    "#]];
    expected.assert_eq(&errors);
}

#[test]
fn it_fails_validation_with_nested_list_type_mismatch() {
    let input = r#"
type Query {
  grid: [[Item]]
}

type Item {
  name: String
}

interface HasGrid {
  grid: [[Item]!]!
}

type MyGrid implements HasGrid {
  grid: [[Item]]
}
"#;
    let errors = Schema::parse_and_validate(input, "schema.graphql")
        .unwrap_err()
        .errors
        .to_string();
    let expected = expect![[r#"
        Error: interface field `HasGrid.grid` expects type `[[Item]!]!` but `MyGrid.grid` of type `[[Item]]` is not a proper subtype
            ╭─[ schema.graphql:15:3 ]
            │
         15 │   grid: [[Item]]
            │   ───────┬──────  
            │          ╰──────── field type is not a proper subtype of `HasGrid.grid`
            │
            ├─[ schema.graphql:15:3 ]
            │
         11 │   grid: [[Item]!]!
            │   ────────┬───────  
            │           ╰───────── `HasGrid.grid` originally defined here
        ────╯
    "#]];
    expected.assert_eq(&errors);
}

#[test]
fn it_fails_validation_when_object_field_missing_interface_argument() {
    let input = r#"
type Query {
  node: Node
}

interface Node {
  field(id: ID!): String
}

type MyNode implements Node {
  field: String
}
"#;
    let errors = Schema::parse_and_validate(input, "schema.graphql")
        .unwrap_err()
        .errors
        .to_string();
    let expected = expect![[r#"
        Error: interface field `Node.field` expects argument `id` but `MyNode.field` does not provide it
            ╭─[ schema.graphql:11:3 ]
            │
         11 │   field: String
            │   ──────┬──────  
            │         ╰──────── missing argument `id` on this field
            │
            ├─[ schema.graphql:11:3 ]
            │
          7 │   field(id: ID!): String
            │         ───┬───  
            │            ╰───── `Node.field(id:)` defined here
        ────╯
    "#]];
    expected.assert_eq(&errors);
}

#[test]
fn it_fails_validation_when_implementation_argument_type_differs() {
    let input = r#"
type Query {
  node: Node
}

interface Node {
  field(id: ID!): String
}

type MyNode implements Node {
  field(id: String!): String
}
"#;
    let errors = Schema::parse_and_validate(input, "schema.graphql")
        .unwrap_err()
        .errors
        .to_string();
    let expected = expect![[r#"
        Error: interface field `Node.field` expects argument `id` of type `ID!` but `MyNode.field` provides type `String!`
            ╭─[ schema.graphql:11:9 ]
            │
         11 │   field(id: String!): String
            │         ─────┬─────  
            │              ╰─────── argument type does not match `Node.field(id:)`
            │
            ├─[ schema.graphql:11:9 ]
            │
          7 │   field(id: ID!): String
            │         ───┬───  
            │            ╰───── `Node.field(id:)` defined here
        ────╯
    "#]];
    expected.assert_eq(&errors);
}

#[test]
fn it_fails_validation_when_implementation_has_extra_required_argument() {
    let input = r#"
type Query {
  node: Node
}

interface Node {
  field(id: ID!): String
}

type MyNode implements Node {
  field(id: ID!, extra: String!): String
}
"#;
    let errors = Schema::parse_and_validate(input, "schema.graphql")
        .unwrap_err()
        .errors
        .to_string();
    let expected = expect![[r#"
        Error: `MyNode.field` has extra required argument `extra` not present in interface `Node.field`
            ╭─[ schema.graphql:11:18 ]
            │
         11 │   field(id: ID!, extra: String!): String
            │                  ───────┬──────  
            │                         ╰──────── required argument `extra` is not in the interface definition
            │
            ├─[ schema.graphql:11:18 ]
            │
          7 │   field(id: ID!): String
            │   ───────────┬──────────  
            │              ╰──────────── `Node.field` defined here
            │ 
            │ Help: Additional arguments on implementing fields must be optional (nullable or have a default value)
        ────╯
    "#]];
    expected.assert_eq(&errors);
}

#[test]
fn it_accepts_implementation_with_extra_optional_argument() {
    let input = r#"
type Query {
  node: Node
}

interface Node {
  field(id: ID!): String
}

type MyNode implements Node {
  field(id: ID!, extra: String): String
}
"#;
    Schema::parse_and_validate(input, "schema.graphql")
        .expect("Expected validation to succeed: extra optional arguments are allowed");
}

#[test]
fn it_accepts_implementation_with_extra_argument_with_default() {
    let input = r#"
type Query {
  node: Node
}

interface Node {
  field(id: ID!): String
}

type MyNode implements Node {
  field(id: ID!, extra: String! = "default"): String
}
"#;
    Schema::parse_and_validate(input, "schema.graphql")
        .expect("Expected validation to succeed: extra arguments with defaults are allowed");
}

#[test]
fn it_fails_validation_when_interface_implementing_interface_has_mismatched_argument() {
    let input = r#"
type Query {
  node: Node
}

interface Node {
  field(id: ID!): String
}

interface Child implements Node {
  field(id: String!): String
}
"#;
    let errors = Schema::parse_and_validate(input, "schema.graphql")
        .unwrap_err()
        .errors
        .to_string();
    let expected = expect![[r#"
        Error: interface field `Node.field` expects argument `id` of type `ID!` but `Child.field` provides type `String!`
            ╭─[ schema.graphql:11:9 ]
            │
         11 │   field(id: String!): String
            │         ─────┬─────  
            │              ╰─────── argument type does not match `Node.field(id:)`
            │
            ├─[ schema.graphql:11:9 ]
            │
          7 │   field(id: ID!): String
            │         ───┬───  
            │            ╰───── `Node.field(id:)` defined here
        ────╯
    "#]];
    expected.assert_eq(&errors);
}

#[test]
fn it_accepts_matching_arguments_on_implementation() {
    let input = r#"
type Query {
  node: Node
}

interface Node {
  field(id: ID!, name: String): String
}

type MyNode implements Node {
  field(id: ID!, name: String): String
}
"#;
    Schema::parse_and_validate(input, "schema.graphql")
        .expect("Expected validation to succeed: matching arguments are valid");
}
