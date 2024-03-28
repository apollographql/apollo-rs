use apollo_compiler::ast::Document;
use apollo_compiler::executable::Selection;
use apollo_compiler::name;
use apollo_compiler::parse_mixed_validate;
use apollo_compiler::schema::ExtendedType;
use apollo_compiler::ExecutableDocument;
use apollo_compiler::Schema;
use std::collections::HashMap;

#[test]
fn it_creates_compiler_from_multiple_sources() {
    let schema = r#"
type Query {
  name: String
  price: Int
  dimensions: Int
  size: Int
  weight: Int
}"#;
    let query = r#"
query ExampleQuery {
  name
  price
  dimensions
  size
  weight
}
      "#;

    let schema = Schema::parse_and_validate(schema, "schema.graphql").unwrap();
    let _doc = ExecutableDocument::parse_and_validate(&schema, query, "query.graphql").unwrap();
}

#[test]
fn it_accesses_operation_definition_parts() {
    let input = r#"
query ExampleQuery($definedVariable: Int, $definedVariable2: Int) {
  topProducts(first: $definedVariable) {
    type
  }
  customer { ... vipCustomer }
}

fragment vipCustomer on User {
  id
  name
  profilePic(size: $definedVariable2)
}

type Query {
  topProducts(first: Int): Product
  customer: User
}

type Product {
  type: String
  price(setPrice: Int): Int
}

type User {
  id: ID
  name: String
  profilePic(size: Int): URL
}

    scalar URL @specifiedBy(url: "https://tools.ietf.org/html/rfc3986")
    "#;

    let (_schema, doc) = parse_mixed_validate(input, "document.graphql").unwrap();

    let operation_names: Vec<_> = doc.named_operations.keys().map(|n| n.as_str()).collect();
    assert_eq!(["ExampleQuery"], operation_names.as_slice());

    let fragment_names: Vec<_> = doc.fragments.keys().map(|n| n.as_str()).collect();
    assert_eq!(["vipCustomer"], fragment_names.as_slice());

    let result = doc.get_operation(Some("ExampleQuery"));
    let operation_variables = match &result {
        Ok(op) => op.variables.iter().map(|var| var.name.as_str()).collect(),
        Err(_) => Vec::new(),
    };
    assert_eq!(
        ["definedVariable", "definedVariable2"],
        operation_variables.as_slice()
    );
}

#[test]
fn it_accesses_fields() {
    let input = r#"
query ExampleQuery {
  name
  price
  dimensions
  size
  weight
}

type Query {
  name: String
  price: Int
  dimensions: Int
  size: Int
  weight: Int
}
"#;

    let (_schema, doc) = parse_mixed_validate(input, "document.graphql").unwrap();

    let op = doc.get_operation(Some("ExampleQuery")).unwrap();
    let field_names: Vec<&str> = op.selection_set.fields().map(|f| f.name.as_str()).collect();
    assert_eq!(
        field_names,
        ["name", "price", "dimensions", "size", "weight"]
    );
}

#[test]
fn it_accesses_inline_fragment_field_types() {
    let input = r#"
query ExampleQuery {
  interface {
    a
    ... on Concrete {
      b
    }
  }

  union {
    ... on Concrete {
      a
      b
    }
  }
}

type Query {
  interface: Interface
  union: Union
}

interface Interface {
  a: String
}

type Concrete implements Interface {
  a: String
  b: Int
  c: Boolean
}

union Union = Concrete
"#;

    let (_schema, doc) = parse_mixed_validate(input, "document.graphql").unwrap();

    let op = doc.get_operation(Some("ExampleQuery")).unwrap();
    let fields: Vec<_> = op.selection_set.fields().collect();

    let interface_field = fields.iter().find(|f| f.name == "interface").unwrap();

    let interface_fields = interface_field.selection_set.fields();
    let interface_selection_fields_types: HashMap<_, _> = interface_fields
        .map(|f| (f.name.as_str(), f.ty().inner_named_type().as_str()))
        .collect();
    assert_eq!(
        interface_selection_fields_types,
        HashMap::from([("a", "String")])
    );

    let inline_fragments: Vec<_> = interface_field
        .selection_set
        .selections
        .iter()
        .filter_map(|sel| sel.as_inline_fragment())
        .collect();

    assert_eq!(inline_fragments.len(), 1);
    let inline_fragment = inline_fragments.first().unwrap();
    assert_eq!(inline_fragment.type_condition, Some(name!("Concrete")));

    let inline_fragment_fields = inline_fragment.selection_set.fields();
    let inline_fragment_fields_types: HashMap<_, _> = inline_fragment_fields
        .map(|f| (f.name.as_str(), f.ty().inner_named_type().as_str()))
        .collect();
    assert_eq!(inline_fragment_fields_types, HashMap::from([("b", "Int")]));

    let union_field = fields.iter().find(|f| f.name == "union").unwrap();

    let union_inline_fragments: Vec<_> = union_field
        .selection_set
        .selections
        .iter()
        .filter_map(|sel| sel.as_inline_fragment())
        .collect();

    assert_eq!(union_inline_fragments.len(), 1);
    let union_inline_fragment = union_inline_fragments.first().unwrap();
    assert_eq!(
        union_inline_fragment.type_condition,
        Some(name!("Concrete"))
    );

    let union_inline_fragment_fields = union_inline_fragment.selection_set.fields();
    let union_inline_fragment_field_types: HashMap<_, _> = union_inline_fragment_fields
        .map(|f| (f.name.as_str(), f.ty().inner_named_type().as_str()))
        .collect();
    assert_eq!(
        union_inline_fragment_field_types,
        HashMap::from([("a", "String"), ("b", "Int"),])
    );
}

#[test]
fn it_accesses_field_definitions_from_operation_definition() {
    let input = r#"
query getProduct {
  size
  topProducts {
    name
    inStock
  }
}

type Query {
  topProducts: Product
  name: String
  size: Int
}

type Product {
  inStock: Boolean @join__field(graph: INVENTORY)
  name: String @join__field(graph: PRODUCTS)
  price: Int
  shippingEstimate: Int
  upc: String!
  weight: Int
}

directive @join__field(graph: join__Graph) on FIELD_DEFINITION
enum join__Graph {
  INVENTORY
  PRODUCTS
}
"#;

    let (_schema, doc) = parse_mixed_validate(input, "document.graphql").unwrap();

    // Get the types of the two top level fields - topProducts and size
    let get_product_op = doc.get_operation(Some("getProduct")).unwrap();
    let op_fields: Vec<_> = get_product_op.selection_set.fields().collect();
    let name_field_def: Vec<_> = op_fields
        .iter()
        .map(|field| field.ty().inner_named_type())
        .collect();
    assert_eq!(name_field_def, ["Int", "Product"]);

    // get the types of the two topProducts selection set fields - name and inStock
    let top_product_fields: Vec<_> = op_fields
        .iter()
        .find(|f| f.name == "topProducts")
        .unwrap()
        .selection_set
        .fields()
        .map(|f| f.ty().inner_named_type())
        .collect();
    assert_eq!(top_product_fields, ["String", "Boolean"]);

    // you can also search for a field in a selection_set and then get its
    // field definition. This looks for topProducts' inStock field's
    // directives.
    let in_stock_directive: Vec<_> = op_fields
        .iter()
        .find(|f| f.name == "topProducts")
        .unwrap()
        .selection_set
        .fields()
        .find(|f| f.name == "inStock")
        .unwrap()
        .definition
        .directives
        .iter()
        .map(|dir| &dir.name)
        .collect();
    assert_eq!(in_stock_directive, ["join__field"]);
}

#[test]
fn it_supports_multiple_independent_queries() {
    let schema = r#"
type Query {
  topProducts: Product
  customer: User
}

type Product {
  type: String
  price(setPrice: Int): Int
}

type User {
  id: ID
  name: String
  profilePic(size: Int): URL
}

scalar URL @specifiedBy(url: "https://tools.ietf.org/html/rfc3986")
"#;

    let product_query = r#"query getProduct { topProducts { type } }"#;
    let customer_query = r#"{ customer { id } }"#;
    let colliding_query = r#"query getProduct { topProducts { type, price } }"#;

    let schema = Schema::parse_and_validate(schema, "schema.graphql").unwrap();
    ExecutableDocument::parse_and_validate(&schema, product_query, "query.graphql").unwrap();
    ExecutableDocument::parse_and_validate(&schema, customer_query, "query.graphql").unwrap();
    ExecutableDocument::parse_and_validate(&schema, colliding_query, "query.graphql").unwrap();
}

#[test]
fn it_accesses_fragment_definition_field_types() {
    let schema = r#"
type Query {
  topProducts: Product
  customer: User
}

type Product {
  type: String
  price(setPrice: Int): Int
}

type User {
  id: ID
  name: String
  profilePic(size: Int): URL
}

scalar URL @specifiedBy(url: "https://tools.ietf.org/html/rfc3986")
"#;
    let query = r#"
query getProduct {
  topProducts {
    type
  }
  customer {
    ... vipCustomer
  }
}

fragment vipCustomer on User {
  id
  name
  profilePic(size: 50)
}
"#;

    let schema = Schema::parse_and_validate(schema, "schema.graphql").unwrap();
    let doc = ExecutableDocument::parse_and_validate(&schema, query, "query.graphql").unwrap();

    let op = doc.get_operation(Some("getProduct")).unwrap();
    let fragment_in_op: Vec<_> = op
        .selection_set
        .fields()
        .find(|field| field.name == "customer")
        .unwrap()
        .selection_set
        .selections
        .iter()
        .filter_map(|sel| sel.as_fragment_spread()?.fragment_def(&doc))
        .collect();
    let field_ty: Vec<_> = fragment_in_op
        .iter()
        .flat_map(|frag| frag.selection_set.fields())
        .map(|f| f.ty().inner_named_type())
        .collect();
    assert_eq!(field_ty, ["ID", "String", "URL"])
}

#[test]
fn it_accesses_schema_operation_types() {
    let input = r#"
schema {
  query: customPetQuery,
}

enum PetType {
  CAT,
  DOG,
}

type customPetQuery {
  name: String,
  age: Int
}

type Subscription {
  changeInPetHousehold: Result
}

type Mutation {
  addPet (name: String!, petType: PetType): Result!
}

type Result {
  id: String
}
"#;

    let (_schema, _doc) = parse_mixed_validate(input, "document.graphql").unwrap();
}

#[test]
fn it_accesses_scalar_definitions() {
    let input = r#"
type Query {
  website: URL,
  amount: Int
}

scalar URL @specifiedBy(url: "https://tools.ietf.org/html/rfc3986")
"#;

    let schema = Schema::parse_and_validate(input, "document.graphql").unwrap();

    let directives: Vec<_> = schema
        .get_scalar("URL")
        .unwrap()
        .directives
        .iter()
        .map(|directive| &directive.name)
        .collect();
    assert_eq!(directives, ["specifiedBy"]);
}

#[test]
fn it_accesses_enum_definitions() {
    let input = r#"
type Query {
  pet: Pet,
}

enum Pet {
    CAT
    DOG
    FOX
}
"#;

    let schema = Schema::parse_and_validate(input, "document.graphql").unwrap();

    let enum_values: Vec<_> = schema.get_enum("Pet").unwrap().values.keys().collect();
    assert_eq!(enum_values, ["CAT", "DOG", "FOX"]);
}

#[test]
fn it_accesses_union_definitions() {
    let input = r#"
schema {
  query: SearchQuery
}

union SearchResult = Photo | Person

type Person {
  name: String
  age: Int
}

type Photo {
  height: Int
  width: Int
}

type SearchQuery {
  firstSearchResult: SearchResult
}
"#;

    let schema = Schema::parse_and_validate(input, "document.graphql").unwrap();

    let union_type = schema.get_union("SearchResult").unwrap();
    let union_members: Vec<_> = union_type.members.iter().collect();
    assert_eq!(union_members, ["Photo", "Person"]);

    let photo_object = schema.get_object("Person").unwrap();

    let fields: Vec<_> = photo_object.fields.keys().collect();
    assert_eq!(fields, ["name", "age"])
}

#[test]
fn it_accesses_directive_definitions() {
    let input = r#"
type Query {
    literature: Book
}

directive @delegateField(name: String!) repeatable on OBJECT | INTERFACE

type Book @delegateField(name: "pageCount") @delegateField(name: "author") {
  id: ID!
}
"#;

    let schema = Schema::parse_and_validate(input, "document.graphql").unwrap();

    let locations: Vec<_> = schema.directive_definitions["delegateField"]
        .locations
        .iter()
        .map(|loc| loc.name())
        .collect();

    assert_eq!(locations, ["OBJECT", "INTERFACE"]);
}

#[test]
fn it_accesses_input_object_definitions() {
    let input = r#"
type Query {
  website: URL,
  amount: Int
}

scalar URL @specifiedBy(url: "https://tools.ietf.org/html/rfc3986")

input Point2D {
  x: Float
  y: Float
}
"#;

    let schema = Schema::parse_and_validate(input, "document.graphql").unwrap();

    let fields: Vec<_> = schema
        .get_input_object("Point2D")
        .unwrap()
        .fields
        .keys()
        .collect();

    assert_eq!(fields, ["x", "y"]);
}

#[test]
fn it_accesses_object_directive_name() {
    let input = r#"
type Query {
  theBook: Book
}

type Book @directiveA(name: "pageCount") @directiveB(name: "author") {
  id: ID!
}

directive @directiveA(name: String) on OBJECT | INTERFACE
directive @directiveB(name: String) on OBJECT | INTERFACE
"#;

    let schema = Schema::parse_and_validate(input, "document.graphql").unwrap();

    let book_obj = schema.get_object("Book").unwrap();

    let directive_names: Vec<_> = book_obj.directives.iter().map(|d| &d.name).collect();
    assert_eq!(directive_names, ["directiveA", "directiveB"]);
}

#[test]
fn it_accesses_object_field_types_directive_name() {
    let input = r#"
type Query {
  me: Person
}

type Person {
  name: String
  picture(size: Number): Url
}

enum Number {
    INT
    FLOAT
}

scalar Url @specifiedBy(url: "https://tools.ietf.org/html/rfc3986")
"#;

    let schema = Schema::parse_and_validate(input, "document.graphql").unwrap();

    let person = schema.get_object("Person").unwrap();

    let field_ty_directive: Vec<_> = person
        .fields
        .values()
        .filter_map(|f| {
            // get access to the actual definition the field is using
            match schema.types.get(f.ty.inner_named_type())? {
                ExtendedType::Scalar(scalar) => Some(scalar),
                _ => None,
            }
        })
        .flat_map(|scalar| {
            // get that definition's directives, for example
            scalar.directives.iter().map(|dir| &dir.name)
        })
        .collect();
    assert_eq!(field_ty_directive, ["specifiedBy"]);

    let field_arg_ty_vals: Vec<_> = person
        .fields
        .values()
        .flat_map(|field| &field.arguments)
        .filter_map(|arg| match schema.types.get(arg.ty.inner_named_type())? {
            ExtendedType::Enum(enum_) => Some(enum_.values.keys()),
            _ => None,
        })
        .flatten()
        .collect();
    assert_eq!(field_arg_ty_vals, ["INT", "FLOAT"])
}

#[test]
fn it_accesses_input_object_field_types_directive_name() {
    let input = r#"
type Query {
  x: Int
}

input Person {
  name: String
  picture: Url
}

scalar Url @specifiedBy(url: "https://tools.ietf.org/html/rfc3986")
"#;

    let schema = Schema::parse_and_validate(input, "document.graphql").unwrap();

    let person = schema.get_input_object("Person").unwrap();

    let field_ty_directive: Vec<_> = person
        .fields
        .values()
        .filter_map(|f| {
            // get access to the actual definition the field is using
            match schema.types.get(f.ty.inner_named_type())? {
                ExtendedType::Scalar(scalar) => Some(scalar),
                _ => None,
            }
        })
        .flat_map(|scalar| {
            // get that definition's directives, for example
            scalar.directives.iter().map(|dir| &dir.name)
        })
        .collect();
    assert_eq!(field_ty_directive, ["specifiedBy"]);
}

#[test]
fn it_accesses_object_defitions() {
    let input = r#"
schema
  @core(feature: "https://specs.apollo.dev/core/v0.1"),
  @core(feature: "https://specs.apollo.dev/join/v0.1")
{
  query: Query
  mutation: Mutation
}

directive @core(feature: String!) repeatable on SCHEMA

directive @join__field(graph: join__Graph, requires: join__FieldSet, provides: join__FieldSet) on FIELD_DEFINITION

directive @join__type(graph: join__Graph!, key: join__FieldSet) repeatable on OBJECT | INTERFACE

directive @join__owner(graph: join__Graph!) on OBJECT | INTERFACE

directive @join__graph(name: String!, url: String!) on ENUM_VALUE

scalar join__FieldSet

enum join__Graph {
  ACCOUNTS @join__graph(name: "accounts" url: "http://localhost:4001")
  INVENTORY @join__graph(name: "inventory" url: "http://localhost:4004")
  PRODUCTS @join__graph(name: "products" url: "http://localhost:4003")
  REVIEWS @join__graph(name: "reviews" url: "http://localhost:4002")
}

type Mutation {
  createProduct(name: String, upc: ID!): Product @join__field(graph: PRODUCTS)
  createReview(body: String, id: ID!, upc: ID!): Review @join__field(graph: REVIEWS)
}

type Product
  @join__owner(graph: PRODUCTS)
  @join__type(graph: PRODUCTS, key: "upc")
  @join__type(graph: INVENTORY, key: "upc")
  @join__type(graph: REVIEWS, key: "upc")
{
  inStock: Boolean @join__field(graph: INVENTORY)
  name: String @join__field(graph: PRODUCTS)
  price: Int @join__field(graph: PRODUCTS)
  reviews: [Review] @join__field(graph: REVIEWS)
  reviewsForAuthor(authorID: ID!): [Review] @join__field(graph: REVIEWS)
  shippingEstimate: Int @join__field(graph: INVENTORY, requires: "price weight")
  upc: String! @join__field(graph: PRODUCTS)
  weight: Int @join__field(graph: PRODUCTS)
}

type Query {
  me: User @join__field(graph: ACCOUNTS)
  topProducts(first: Int = 5): [Product] @join__field(graph: PRODUCTS)
}

type Review
  @join__owner(graph: REVIEWS)
  @join__type(graph: REVIEWS, key: "id")
{
  author: User @join__field(graph: REVIEWS, provides: "username")
  body: String @join__field(graph: REVIEWS)
  id: ID! @join__field(graph: REVIEWS)
  product: Product @join__field(graph: REVIEWS)
}

type User
  @join__owner(graph: ACCOUNTS)
  @join__type(graph: ACCOUNTS, key: "id")
  @join__type(graph: REVIEWS, key: "id")
{
  id: ID! @join__field(graph: ACCOUNTS)
  name: String @join__field(graph: ACCOUNTS)
  reviews: [Review] @join__field(graph: REVIEWS)
  username: String @join__field(graph: ACCOUNTS)
}
"#;

    let schema = Schema::parse_and_validate(input, "document.graphql").unwrap();

    let object_names: Vec<_> = schema
        .types
        .iter()
        .filter(|(_name, def)| def.is_object() && !def.is_built_in())
        .map(|(name, _def)| name)
        .collect();
    assert_eq!(
        object_names,
        ["Mutation", "Product", "Query", "Review", "User"],
    );
}

#[test]
fn it_can_access_root_db_in_thread() {
    let input = r#"
type Query {
  website: URL,
  amount: Int
}

scalar URL @specifiedBy(url: "https://tools.ietf.org/html/rfc3986")
"#;

    let schema = Schema::parse_and_validate(input, "document.graphql").unwrap();

    let (result1, result2) = std::thread::scope(|scope| {
        let thread1 = scope.spawn(|| {
            assert!(schema.get_object("Query").is_some());
        });
        let thread2 = scope.spawn(|| {
            assert_eq!(
                schema
                    .types
                    .values()
                    .filter(|ty| ty.is_scalar() && !ty.is_built_in())
                    .count(),
                1
            );
        });
        (thread1.join(), thread2.join())
    });

    // Neither thread panicked
    result1.expect("get_object return None");
    result2.expect("scalars failed");
}

#[test]
fn precomputed_schema_can_multi_thread() {
    let sdl = r#"
type Query {
    website: URL,
    amount: Int
}
scalar URL
"#;
    let query = "{ website }";

    let schema = Schema::parse_and_validate(sdl, "schema.graphql").unwrap();

    let results = std::thread::scope(|scope| {
        let handles: Vec<_> = (0..2)
            .map(|_| {
                scope.spawn(|| {
                    let document =
                        ExecutableDocument::parse_and_validate(&schema, query, "query.graphql")
                            .unwrap();
                    let selections = &document
                        .anonymous_operation
                        .as_ref()?
                        .selection_set
                        .selections;

                    match selections.first()? {
                        Selection::Field(field) => {
                            Some(field.definition.ty.inner_named_type().to_string())
                        }
                        _ => None,
                    }
                })
            })
            .collect();
        handles.into_iter().map(|h| h.join()).collect::<Vec<_>>()
    });

    assert_eq!(results.len(), 2);
    for result in results {
        assert_eq!(result.unwrap().as_deref(), Some("URL"));
    }
}

#[test]
fn initial_indent() {
    let ast = Document::parse("query Op { field ... @defer { otherField }}", "").unwrap();
    let formatted = format!(
        "Request:\n  operationName: {}\n  query:\n{}",
        "Op",
        ast.serialize().initial_indent_level(2)
    );
    let expected = expect_test::expect![[r#"
        Request:
          operationName: Op
          query:
            query Op {
              field
              ... @defer {
                otherField
              }
            }
    "#]];
    expected.assert_eq(&formatted);
}
