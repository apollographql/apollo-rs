use apollo_compiler::Schema;
use apollo_parser::Parser;
use apollo_smith::{Document, DocumentBuilder};
use libfuzzer_sys::arbitrary::{Result, Unstructured};

/// This generate an arbitrary valid GraphQL document
pub fn generate_valid_document(input: &[u8]) -> Result<String> {
    drop(env_logger::try_init());

    let mut u = Unstructured::new(input);
    let gql_doc = DocumentBuilder::new(&mut u)?;
    let document = gql_doc.finish();

    Ok(document.into())
}

/// Log the error and the document generated for these errors
/// Save it into files
pub fn log_gql_doc(gql_doc: &str, errors: &str) {
    log::debug!("writing test case to test.graphql ...");
    std::fs::write("test_case.graphql", gql_doc).unwrap();
    std::fs::write("test_case_error.log", errors).unwrap();
}

pub fn generate_valid_operation(input: &[u8]) -> Result<(String, String)> {
    drop(env_logger::try_init());

    let ts = Parser::new(SUPERGRAPH).parse();

    let mut u = Unstructured::new(input);
    let mut doc = DocumentBuilder::with_document(
        &mut u,
        Document::try_from(ts.document()).expect("document should not have errors"),
    )?;

    let operation_def: String = doc.operation_definition()?.unwrap().into();
    let doc: String = doc.finish().into();

    Ok((operation_def, doc))
}

const SUPERGRAPH: &'static str = r#"
schema @core(feature: "https://specs.apollo.dev/core/v0.1"){
  query: Query
  mutation: Mutation
}
extend schema @core(feature: "https://specs.apollo.dev/join/v0.1") 
directive @core(feature: String!) repeatable on SCHEMA

directive @join__field(
  graph: join__Graph
  requires: join__FieldSet
  provides: join__FieldSet
) on FIELD_DEFINITION

directive @join__type(
  graph: join__Graph!
  key: join__FieldSet
) repeatable on OBJECT | INTERFACE

directive @join__owner(graph: join__Graph!) on OBJECT | INTERFACE

directive @join__graph(name: String!, url: String!) on ENUM_VALUE

# Uncomment if you want to reproduce the bug with the order of skip/include directives
# directive @skip(if: Boolean!) on FIELD | FRAGMENT_SPREAD | INLINE_FRAGMENT
# directive @include(if: Boolean!) on FIELD | FRAGMENT_SPREAD | INLINE_FRAGMENT

scalar join__FieldSet @specifiedBy(url: "example.com")

enum join__Graph {
  ACCOUNTS @join__graph(name: "accounts", url: "http://subgraphs:4001/graphql")
  INVENTORY
    @join__graph(name: "inventory", url: "http://subgraphs:4004/graphql")
  PRODUCTS @join__graph(name: "products", url: "http://subgraphs:4003/graphql")
  REVIEWS @join__graph(name: "reviews", url: "http://subgraphs:4002/graphql")
}

type Mutation {
  createProduct(name: String, upc: ID!): Product @join__field(graph: PRODUCTS)
  createReview(body: String, id: ID!, upc: ID!): Review
    @join__field(graph: REVIEWS)
}

type Product
  @join__owner(graph: PRODUCTS)
  @join__type(graph: PRODUCTS, key: "upc")
  @join__type(graph: INVENTORY, key: "upc")
  @join__type(graph: REVIEWS, key: "upc") {
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
  @join__type(graph: REVIEWS, key: "id") {
  author: User @join__field(graph: REVIEWS, provides: "username")
  body: String @join__field(graph: REVIEWS)
  id: ID! @join__field(graph: REVIEWS)
  product: Product @join__field(graph: REVIEWS)
}

type User
  @join__owner(graph: ACCOUNTS)
  @join__type(graph: ACCOUNTS, key: "id")
  @join__type(graph: REVIEWS, key: "id") {
  id: ID! @join__field(graph: ACCOUNTS)
  name: String @join__field(graph: ACCOUNTS)
  reviews: [Review] @join__field(graph: REVIEWS)
  username: String @join__field(graph: ACCOUNTS)
}

"#;
