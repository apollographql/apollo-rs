use apollo_compiler::executable::Selection;
use apollo_compiler::hir;
use apollo_compiler::hir::TypeDefinition;
use apollo_compiler::ApolloCompiler;
use apollo_compiler::Arc;
use apollo_compiler::HirDatabase;
use apollo_compiler::ReprDatabase;
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

    let mut compiler = ApolloCompiler::new();
    compiler.add_document(schema, "schema.graphql");
    compiler.add_executable(query, "query.graphql");
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

    let mut compiler = ApolloCompiler::new();
    let document_id = compiler.add_document(input, "document.graphql");

    let diagnostics = compiler.validate();
    for diagnostic in &diagnostics {
        println!("{diagnostic}");
    }
    assert!(diagnostics.is_empty());

    let operations = compiler.db.operations(document_id);
    let operation_names: Vec<_> = operations.iter().filter_map(|op| op.name()).collect();
    assert_eq!(["ExampleQuery"], operation_names.as_slice());

    let fragments = compiler.db.fragments(document_id);
    let fragment_names: Vec<_> = fragments.keys().map(|name| &**name).collect();
    assert_eq!(["vipCustomer"], fragment_names.as_slice());

    let operation_variables: Vec<String> = match operations
        .iter()
        .find(|op| op.name() == Some("ExampleQuery"))
    {
        Some(op) => op
            .variables()
            .iter()
            .map(|var| var.name().to_string())
            .collect(),
        None => Vec::new(),
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

    let mut compiler = ApolloCompiler::new();
    let document_id = compiler.add_document(input, "document.graphql");

    let diagnostics = compiler.validate();
    for diagnostic in &diagnostics {
        println!("{diagnostic}");
    }
    assert!(diagnostics.is_empty());

    let operations = compiler.db.operations(document_id);
    let fields = operations
        .iter()
        .find(|op| op.name() == Some("ExampleQuery"))
        .unwrap()
        .fields(&compiler.db);
    let field_names: Vec<&str> = fields.iter().map(|f| f.name()).collect();
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

    let mut compiler = ApolloCompiler::new();
    let document_id = compiler.add_document(input, "document.graphql");

    let diagnostics = compiler.validate();
    assert!(diagnostics.is_empty());

    let operations = compiler.db.operations(document_id);
    let fields = operations
        .iter()
        .find(|op| op.name() == Some("ExampleQuery"))
        .unwrap()
        .fields(&compiler.db);

    let interface_field = fields.iter().find(|f| f.name() == "interface").unwrap();

    let interface_fields = interface_field.selection_set().fields();
    let interface_selection_fields_types: HashMap<_, _> = interface_fields
        .iter()
        .map(|f| (f.name(), f.ty(&compiler.db).map(|f| f.name())))
        .collect();
    assert_eq!(
        interface_selection_fields_types,
        HashMap::from([("a", Some("String".to_string()))])
    );

    let inline_fragments: Vec<_> = interface_field.selection_set().inline_fragments();

    assert_eq!(inline_fragments.len(), 1);
    let inline_fragment = inline_fragments.first().unwrap();
    assert_eq!(inline_fragment.type_condition(), Some("Concrete"));

    let inline_fragment_fields = inline_fragment.selection_set().fields();
    let inline_fragment_fields_types: HashMap<_, _> = inline_fragment_fields
        .iter()
        .map(|f| (f.name(), f.ty(&compiler.db).map(|ty| ty.name())))
        .collect();
    assert_eq!(
        inline_fragment_fields_types,
        HashMap::from([("b", Some("Int".to_string()))])
    );

    let union_field = fields.iter().find(|f| f.name() == "union").unwrap();

    let union_inline_fragments: Vec<_> = union_field.selection_set().inline_fragments();

    assert_eq!(union_inline_fragments.len(), 1);
    let union_inline_fragment = union_inline_fragments.first().unwrap();
    assert_eq!(union_inline_fragment.type_condition(), Some("Concrete"));

    let union_inline_fragment_fields = union_inline_fragment.selection_set().fields();
    let union_inline_fragment_field_types: HashMap<_, _> = union_inline_fragment_fields
        .iter()
        .map(|f| (f.name(), f.ty(&compiler.db).map(|ty| ty.name())))
        .collect();
    assert_eq!(
        union_inline_fragment_field_types,
        HashMap::from([
            ("a", Some("String".to_string())),
            ("b", Some("Int".to_string())),
        ])
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

    let mut compiler = ApolloCompiler::new();
    let document_id = compiler.add_document(input, "document.graphql");

    let diagnostics = compiler.validate();
    for diagnostic in &diagnostics {
        println!("{diagnostic}");
    }
    assert!(diagnostics.is_empty());

    // Get the types of the two top level fields - topProducts and size
    let operations = compiler.db.operations(document_id);
    let get_product_op = operations
        .iter()
        .find(|op| op.name() == Some("getProduct"))
        .unwrap();
    let op_fields = get_product_op.fields(&compiler.db);
    let name_field_def: Vec<String> = op_fields
        .iter()
        .filter_map(|field| Some(field.ty(&compiler.db)?.name()))
        .collect();
    assert_eq!(name_field_def, ["Int", "Product"]);

    // get the types of the two topProducts selection set fields - name and inStock
    let top_products = op_fields
        .iter()
        .find(|f| f.name() == "topProducts")
        .unwrap()
        .selection_set()
        .fields();

    let top_product_fields: Vec<String> = top_products
        .iter()
        .filter_map(|f| Some(f.ty(&compiler.db)?.name()))
        .collect();
    assert_eq!(top_product_fields, ["String", "Boolean"]);

    // you can also search for a field in a selection_set and then get its
    // field definition. This looks for topProducts' inStock field's
    // directives.
    let in_stock_field = op_fields
        .iter()
        .find(|f| f.name() == "topProducts")
        .unwrap()
        .selection_set()
        .field("inStock")
        .unwrap()
        .field_definition(&compiler.db)
        .unwrap();
    let in_stock_directive: Vec<&str> = in_stock_field
        .directives()
        .iter()
        .map(|dir| dir.name())
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

    let mut compiler = ApolloCompiler::new();
    compiler.add_type_system(schema, "schema.graphql");
    compiler.add_executable(product_query, "query.graphql");
    compiler.add_executable(customer_query, "query.graphql");
    compiler.add_executable(colliding_query, "query.graphql");

    assert_eq!(compiler.validate(), &[]);
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

    let mut compiler = ApolloCompiler::new();
    compiler.add_type_system(schema, "schema.graphql");
    let query_id = compiler.add_executable(query, "query.graphql");

    let diagnostics = compiler.validate();
    for diagnostic in &diagnostics {
        println!("{diagnostic}");
    }
    assert!(diagnostics.is_empty());

    let op = compiler
        .db
        .find_operation(query_id, Some("getProduct".into()));
    let fragment_in_op: Vec<hir::FragmentDefinition> = op
        .unwrap()
        .fields(&compiler.db)
        .iter()
        .find(|field| field.name() == "customer")
        .unwrap()
        .selection_set()
        .selection()
        .iter()
        .filter_map(|sel| match sel {
            hir::Selection::FragmentSpread(frag) => {
                Some(frag.fragment(&compiler.db)?.as_ref().clone())
            }
            _ => None,
        })
        .collect();
    let fragment_fields: Vec<hir::Field> = fragment_in_op
        .iter()
        .flat_map(|frag| frag.selection_set().fields())
        .collect();
    let field_ty: Vec<String> = fragment_fields
        .iter()
        .filter_map(|f| Some(f.ty(&compiler.db)?.name()))
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

    let mut compiler = ApolloCompiler::new();
    compiler.add_document(input, "document.graphql");

    let diagnostics = compiler.validate();
    for diagnostic in &diagnostics {
        println!("{diagnostic}");
    }
    assert!(diagnostics.is_empty());
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

    let mut compiler = ApolloCompiler::new();
    compiler.add_document(input, "document.graphql");

    let diagnostics = compiler.validate();
    for diagnostic in &diagnostics {
        println!("{diagnostic}");
    }
    assert!(diagnostics.is_empty());

    let scalars = compiler.db.scalars();

    let directives: Vec<&str> = scalars["URL"]
        .self_directives()
        .iter()
        .map(|directive| directive.name())
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

    let mut compiler = ApolloCompiler::new();
    compiler.add_document(input, "document.graphql");

    let diagnostics = compiler.validate();
    for diagnostic in &diagnostics {
        println!("{diagnostic}");
    }
    assert!(diagnostics.is_empty());

    let enums = compiler.db.enums();
    let enum_values: Vec<&str> = enums["Pet"]
        .self_values()
        .iter()
        .map(|enum_val| enum_val.enum_value())
        .collect();
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

    let mut compiler = ApolloCompiler::new();
    compiler.add_document(input, "document.graphql");

    let diagnostics = compiler.validate();
    for diagnostic in &diagnostics {
        println!("{diagnostic}");
    }
    assert!(diagnostics.is_empty());

    let unions = compiler.db.unions();
    let union_members: Vec<&str> = unions["SearchResult"]
        .self_members()
        .iter()
        .map(|member| member.name())
        .collect();
    assert_eq!(union_members, ["Photo", "Person"]);

    let photo_object = unions["SearchResult"]
        .self_members()
        .iter()
        .find(|mem| mem.name() == "Person")
        .unwrap()
        .object(&compiler.db);

    if let Some(photo) = photo_object {
        let fields: Vec<&str> = photo
            .self_fields()
            .iter()
            .map(|field| field.name())
            .collect();
        assert_eq!(fields, ["name", "age"])
    }
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

    let mut compiler = ApolloCompiler::new();
    compiler.add_document(input, "document.graphql");

    let diagnostics = compiler.validate();
    for diagnostic in &diagnostics {
        println!("{diagnostic}");
    }
    assert!(diagnostics.is_empty());

    let directives = compiler.db.directive_definitions();
    let locations: Vec<_> = directives["delegateField"]
        .directive_locations()
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

    let mut compiler = ApolloCompiler::new();
    compiler.add_document(input, "document.graphql");

    let diagnostics = compiler.validate();
    for diagnostic in &diagnostics {
        println!("{diagnostic}");
    }
    assert!(diagnostics.is_empty());

    let input_objects = compiler.db.input_objects();
    let fields: Vec<&str> = input_objects["Point2D"]
        .self_fields()
        .iter()
        .map(|val| val.name())
        .collect();

    assert_eq!(fields, ["x", "y"]);
}

#[test]
fn it_accesses_object_directive_name() {
    let input = r#"

type Book @directiveA(name: "pageCount") @directiveB(name: "author") {
  id: ID!
}

directive @directiveA(name: String) on OBJECT | INTERFACE
directive @directiveB(name: String) on OBJECT | INTERFACE
"#;

    let mut compiler = ApolloCompiler::new();
    compiler.add_document(input, "document.graphql");

    let diagnostics = compiler.validate();
    for diagnostic in &diagnostics {
        println!("{diagnostic}");
    }
    assert!(diagnostics.is_empty());

    let book_obj = compiler
        .db
        .find_object_type_by_name("Book".to_string())
        .unwrap();

    let directive_names: Vec<&str> = book_obj
        .self_directives()
        .iter()
        .map(|d| d.name())
        .collect();
    assert_eq!(directive_names, ["directiveA", "directiveB"]);
}

#[test]
fn it_accesses_object_field_types_directive_name() {
    let input = r#"
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

    let mut compiler = ApolloCompiler::new();
    compiler.add_document(input, "document.graphql");

    let diagnostics = compiler.validate();
    for diagnostic in &diagnostics {
        println!("{diagnostic}");
    }
    assert!(diagnostics.is_empty());

    let person_obj = compiler.db.find_object_type_by_name("Person".to_string());

    if let Some(person) = person_obj {
        let field_ty_directive: Vec<String> = person
            .self_fields()
            .iter()
            .filter_map(|f| {
                // get access to the actual definition the field is using
                if let Some(field_ty) = f.ty().type_def(&compiler.db) {
                    match field_ty {
                        // get that definition's directives, for example
                        TypeDefinition::ScalarTypeDefinition(scalar) => {
                            let dir_names: Vec<String> = scalar
                                .self_directives()
                                .iter()
                                .map(|dir| dir.name().to_owned())
                                .collect();
                            return Some(dir_names);
                        }
                        _ => return None,
                    }
                }
                None
            })
            .flatten()
            .collect();
        assert_eq!(field_ty_directive, ["specifiedBy"]);

        let field_arg_ty_vals: Vec<String> = person
            .self_fields()
            .iter()
            .flat_map(|f| {
                let enum_vals: Vec<String> = f
                    .arguments()
                    .input_values()
                    .iter()
                    .filter_map(|val| {
                        if let Some(input_ty) = val.ty().type_def(&compiler.db) {
                            match input_ty {
                                // get that definition's directives, for example
                                TypeDefinition::EnumTypeDefinition(enum_) => {
                                    let dir_names: Vec<String> = enum_
                                        .self_values()
                                        .iter()
                                        .map(|enum_val| enum_val.enum_value().to_owned())
                                        .collect();
                                    return Some(dir_names);
                                }
                                _ => return None,
                            }
                        }
                        None
                    })
                    .flatten()
                    .collect();
                enum_vals
            })
            .collect();
        assert_eq!(field_arg_ty_vals, ["INT", "FLOAT"])
    }
}

#[test]
fn it_accesses_input_object_field_types_directive_name() {
    let input = r#"
input Person {
  name: String
  picture: Url
}

scalar Url @specifiedBy(url: "https://tools.ietf.org/html/rfc3986")
"#;

    let mut compiler = ApolloCompiler::new();
    compiler.add_document(input, "document.graphql");

    let diagnostics = compiler.validate();
    for diagnostic in &diagnostics {
        println!("{diagnostic}");
    }
    assert!(diagnostics.is_empty());

    let person_obj = compiler.db.find_input_object_by_name("Person".to_string());

    if let Some(person) = person_obj {
        let field_ty_directive: Vec<String> = person
            .self_fields()
            .iter()
            .filter_map(|f| {
                if let Some(field_ty) = f.ty().type_def(&compiler.db) {
                    match field_ty {
                        TypeDefinition::ScalarTypeDefinition(scalar) => {
                            let dir_names: Vec<String> = scalar
                                .self_directives()
                                .iter()
                                .map(|dir| dir.name().to_owned())
                                .collect();
                            return Some(dir_names);
                        }
                        _ => return None,
                    }
                }
                None
            })
            .flatten()
            .collect();
        assert_eq!(field_ty_directive, ["specifiedBy"]);
    }
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

    let mut compiler = ApolloCompiler::new();
    compiler.add_document(input, "document.graphql");

    let diagnostics = compiler.validate();
    for diagnostic in &diagnostics {
        println!("{diagnostic}");
    }
    // the scalar warning diagnostic
    assert_eq!(diagnostics.len(), 1);

    let object_types = compiler.db.object_types();
    let object_names: Vec<_> = object_types.keys().map(|name| &**name).collect();
    assert_eq!(
        ["Mutation", "Product", "Query", "Review", "User"],
        object_names.as_slice()
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

    let mut compiler = ApolloCompiler::new();
    compiler.add_document(input, "document.graphql");

    let diagnostics = compiler.validate();
    for diagnostic in &diagnostics {
        println!("{diagnostic}");
    }

    assert!(diagnostics.is_empty());

    let snapshot = compiler.snapshot();
    let snapshot2 = compiler.snapshot();

    let thread1 = std::thread::spawn(move || snapshot.find_object_type_by_name("Query".into()));
    let thread2 = std::thread::spawn(move || snapshot2.scalars());

    thread1.join().expect("object_type_by_name panicked");
    thread2.join().expect("scalars failed");
}

#[test]
fn inputs_can_be_updated() {
    let input = r#"
type Query {
  website: URL,
  amount: Int
}
"#;

    let mut compiler = ApolloCompiler::new();
    let input_id = compiler.add_document(input, "document.graphql");

    let object_type = compiler
        .db
        .find_object_type_by_name("Query".into())
        .unwrap();
    assert!(object_type.self_directives().is_empty());

    let input = r#"
type Query @withDirective {
  website: URL,
  amount: Int
}
"#;
    compiler.update_document(input_id, input);

    let object_type = compiler
        .db
        .find_object_type_by_name("Query".into())
        .unwrap();
    assert_eq!(object_type.self_directives().len(), 1);
}

#[test]
fn old_precomputed_schema_can_multi_thread() {
    let schema = r#"
type Query {
    website: URL,
    amount: Int
}
"#;
    let query = "{ website }";

    let mut compiler = ApolloCompiler::new();
    compiler.add_type_system(schema, "schema.graphql");
    let type_system = compiler.db.type_system();

    let handles: Vec<_> = (0..2)
        .map(|_| {
            let cloned = Arc::clone(&type_system); // cheap refcount increment
            std::thread::spawn(move || {
                let mut compiler = ApolloCompiler::new();
                let query_id = compiler.add_executable(query, "query.graphql");
                compiler.set_type_system_hir(cloned);
                compiler
                    .db
                    .find_operation(query_id, None)
                    .unwrap()
                    .fields(&compiler.db)[0]
                    .ty(&compiler.db)
                    .unwrap()
                    .name()
            })
        })
        .collect();
    assert_eq!(handles.len(), 2);
    for handle in handles {
        assert_eq!(handle.join().unwrap(), "URL");
    }
}

#[test]
fn precomputed_schema_can_multi_thread() {
    let sdl = r#"
type Query {
    website: URL,
    amount: Int
}
"#;
    let query = "{ website }";

    let mut compiler = ApolloCompiler::new();
    compiler.add_type_system(sdl, "schema.graphql");
    let schema = compiler.db.schema();

    let handles: Vec<_> = (0..2)
        .map(|_| {
            let cloned = Arc::clone(&schema); // cheap refcount increment
            std::thread::spawn(move || {
                let mut compiler = ApolloCompiler::from_schema(cloned);
                let query_id = compiler.add_executable(query, "query.graphql");
                let document = compiler.db.executable_document(query_id);
                let selections = &document
                    .anonymous_operation
                    .as_ref()?
                    .selection_set
                    .selections;

                match selections.get(0)? {
                    Selection::Field(field) => {
                        Some(field.definition.ty.inner_named_type().to_string())
                    }
                    _ => None,
                }
            })
        })
        .collect();
    assert_eq!(handles.len(), 2);
    for handle in handles {
        assert_eq!(handle.join().unwrap().as_deref(), Some("URL"));
    }
}

#[test]
fn validate_with_precomputed_schema() {
    let schema = r#"
          schema {
            query: Query
          }

          type Query {
            peopleCount: Int!
            person: Person!
          }

          interface Pet {
            name: String!
          }

          type Dog implements Pet {
            name: String!
            dogBreed: DogBreed!
          }

          type Cat implements Pet {
            name: String!
            catBreed: CatBreed!
          }

          type Person {
            firstName: String!
            lastName: String!
            age: Int
            pets: [Pet!]!
          }

          enum DogBreed {
            CHIHUAHUA
            RETRIEVER
            LAB
          }

          enum CatBreed {
            TABBY
            MIX
          }
        "#;

    let mut compiler = ApolloCompiler::new();
    compiler.add_type_system(schema, "schema.graphql");

    let type_system = compiler.db.schema();
    let mut another_compiler = ApolloCompiler::from_schema(type_system);

    let file_id = another_compiler.add_executable("{ person { pets { name } } }", "query.graphql");
    for diag in another_compiler.validate() {
        println!("{diag}");
    }
    assert!(another_compiler.validate().is_empty());

    another_compiler.update_executable(file_id, "{ person { pets { dogBreed } } }");
    for diag in another_compiler.validate() {
        println!("{diag}");
    }
    assert_eq!(another_compiler.validate().len(), 1);
}
