mod diagnostics;
mod queries;
#[cfg(test)]
mod tests;
mod validation;

use std::sync::Arc;

use apollo_parser::{ast, SyntaxTree};
pub use queries::{
    database::{Database, SourceDatabase},
    values,
};

use diagnostics::ApolloDiagnostic;
use validation::Validator;

pub struct ApolloCompiler {
    pub db: Database,
}

impl ApolloCompiler {
    pub fn new(input: &str) -> Self {
        let mut db = Database::default();
        let input = input.to_string();
        db.set_input_string((), Arc::new(input));
        Self { db }
    }

    pub fn parse(&self) -> Arc<SyntaxTree> {
        self.db.parse()
    }

    // should probably return an iter here
    pub fn validate(&self) -> Vec<ApolloDiagnostic> {
        let mut validator = Validator::new(&self.db);
        validator.validate().into()
    }

    pub fn syntax_errors(&self) -> Vec<ApolloDiagnostic> {
        self.db.syntax_errors()
    }

    pub fn definitions(&self) -> Arc<Vec<ast::Definition>> {
        self.db.definitions()
    }

    pub fn operations(&self) -> Arc<Vec<values::OperationDefinition>> {
        self.db.operations()
    }

    pub fn fragments(&self) -> Arc<Vec<values::FragmentDefinition>> {
        self.db.fragments()
    }

    pub fn schema(&self) -> Arc<values::SchemaDefinition> {
        self.db.schema()
    }

    pub fn object_types(&self) -> Arc<Vec<values::ObjectTypeDefinition>> {
        self.db.object_types()
    }

    pub fn scalars(&self) -> Arc<Vec<values::ScalarTypeDefinition>> {
        self.db.scalars()
    }

    pub fn enums(&self) -> Arc<Vec<values::EnumTypeDefinition>> {
        self.db.enums()
    }

    pub fn unions(&self) -> Arc<Vec<values::UnionTypeDefinition>> {
        self.db.unions()
    }

    pub fn directive_definitions(&self) -> Arc<Vec<values::DirectiveDefinition>> {
        self.db.directive_definitions()
    }

    pub fn input_objects(&self) -> Arc<Vec<values::InputObjectTypeDefinition>> {
        self.db.input_objects()
    }
}

#[cfg(test)]
mod test {
    use crate::{values::Definition, ApolloCompiler, SourceDatabase};

    #[test]
    fn it_accesses_operation_definition_parts() {
        let input = r#"
query ExampleQuery($definedVariable: Int, $definedVariable2: Boolean) {
  topProducts(first: $definedVariable) {
    name
  }
  ... vipCustomer
}

fragment vipCustomer on User {
  id
  name
  profilePic(size: 50)
  status(activity: $definedVariable2)
}

type Query {
    topProducts: Product
}

type Product {
  name: String
  price(setPrice: Int): Int
}
"#;

        let ctx = ApolloCompiler::new(input);
        let diagnostics = ctx.validate();
        for diagnostic in &diagnostics {
            println!("{}", diagnostic);
        }
        assert!(diagnostics.is_empty());

        let operations = ctx.operations();
        let operation_names: Vec<_> = operations.iter().filter_map(|op| op.name()).collect();
        assert_eq!(["ExampleQuery"], operation_names.as_slice());

        let fragments = ctx.fragments();
        let fragment_names: Vec<_> = fragments.iter().map(|fragment| fragment.name()).collect();
        assert_eq!(["vipCustomer"], fragment_names.as_slice());

        let operation_variables: Vec<String> = match operations
            .iter()
            .find(|op| op.name() == Some("ExampleQuery"))
        {
            Some(op) => op.variables().iter().map(|var| var.name.clone()).collect(),
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

        let ctx = ApolloCompiler::new(input);
        let diagnostics = ctx.validate();
        for diagnostic in &diagnostics {
            println!("{}", diagnostic);
        }
        assert!(diagnostics.is_empty());

        let operations = ctx.operations();
        let fields = operations
            .iter()
            .find(|op| op.name() == Some("ExampleQuery"))
            .unwrap()
            .fields(&ctx.db);
        let field_names: Vec<&str> = fields.iter().map(|f| f.name()).collect();
        assert_eq!(
            field_names,
            ["name", "price", "dimensions", "size", "weight"]
        );
    }

    #[test]
    fn it_accesses_schema_operation_types() {
        let input = r#"
schema {
  query: customPetQuery,
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

        let ctx = ApolloCompiler::new(input);
        let diagnostics = ctx.validate();
        for diagnostic in &diagnostics {
            println!("{}", diagnostic);
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

        let ctx = ApolloCompiler::new(input);
        let diagnostics = ctx.validate();
        for diagnostic in &diagnostics {
            println!("{}", diagnostic);
        }
        assert!(diagnostics.is_empty());

        let scalars = ctx.scalars();

        let directives: Vec<&str> = scalars
            .iter()
            .find(|scalar| scalar.name() == "URL")
            .unwrap()
            .directives()
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

        let ctx = ApolloCompiler::new(input);
        let diagnostics = ctx.validate();
        for diagnostic in &diagnostics {
            println!("{}", diagnostic);
        }
        assert!(diagnostics.is_empty());

        let enums = ctx.enums();
        let enum_values: Vec<&str> = enums
            .iter()
            .find(|enum_def| enum_def.name() == "Pet")
            .unwrap()
            .enum_values_definition()
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

        let ctx = ApolloCompiler::new(input);
        let diagnostics = ctx.validate();
        for diagnostic in &diagnostics {
            println!("{}", diagnostic);
        }
        assert!(diagnostics.is_empty());

        let unions = ctx.unions();
        let union_members: Vec<&str> = unions
            .iter()
            .find(|def| def.name() == "SearchResult")
            .unwrap()
            .union_members()
            .iter()
            .map(|member| member.name())
            .collect();
        assert_eq!(union_members, ["Photo", "Person"]);

        let photo_object = unions
            .iter()
            .find(|def| def.name() == "SearchResult")
            .unwrap()
            .union_members()
            .iter()
            .find(|mem| mem.name() == "Person")
            .unwrap()
            .object(&ctx.db);

        if let Some(photo) = photo_object {
            let fields: Vec<&str> = photo
                .fields_definition()
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

        let ctx = ApolloCompiler::new(input);
        let diagnostics = ctx.validate();
        for diagnostic in &diagnostics {
            println!("{}", diagnostic);
        }
        assert!(diagnostics.is_empty());

        let directives = ctx.directive_definitions();
        let locations: Vec<String> = directives
            .iter()
            .filter_map(|dir| {
                if dir.name() == "delegateField" {
                    let locations: Vec<String> = dir
                        .directive_locations()
                        .iter()
                        .map(|loc| loc.clone().into())
                        .collect();
                    Some(locations)
                } else {
                    None
                }
            })
            .flatten()
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

        let ctx = ApolloCompiler::new(input);
        let diagnostics = ctx.validate();
        for diagnostic in &diagnostics {
            println!("{}", diagnostic);
        }
        assert!(diagnostics.is_empty());

        let input_objects = ctx.input_objects();
        let fields: Vec<&str> = input_objects
            .iter()
            .filter_map(|input| {
                if input.name() == "Point2D" {
                    let fields: Vec<&str> = input
                        .input_fields_definition()
                        .iter()
                        .map(|val| val.name())
                        .collect();
                    Some(fields)
                } else {
                    None
                }
            })
            .flatten()
            .collect();

        assert_eq!(fields, ["x", "y"]);
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

        let ctx = ApolloCompiler::new(input);
        let diagnostics = ctx.validate();
        for diagnostic in &diagnostics {
            println!("{}", diagnostic);
        }
        assert!(diagnostics.is_empty());

        let person_obj = ctx.db.find_object_type_by_name("Person".to_string());

        if let Some(person) = person_obj {
            let field_ty_directive: Vec<String> = person
                .fields_definition()
                .iter()
                .filter_map(|f| {
                    // get access to the actual definition the field is using
                    if let Some(field_ty) = f.ty().ty(&ctx.db) {
                        match field_ty.as_ref() {
                            // get that definition's directives, for example
                            Definition::ScalarTypeDefinition(scalar) => {
                                let dir_names: Vec<String> = scalar
                                    .directives()
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
                .fields_definition()
                .iter()
                .flat_map(|f| {
                    let enum_vals: Vec<String> = f
                        .arguments()
                        .input_values()
                        .iter()
                        .filter_map(|val| {
                            if let Some(input_ty) = val.ty().ty(&ctx.db) {
                                match input_ty.as_ref() {
                                    // get that definition's directives, for example
                                    Definition::EnumTypeDefinition(enum_) => {
                                        let dir_names: Vec<String> = enum_
                                            .enum_values_definition()
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

        let ctx = ApolloCompiler::new(input);
        let diagnostics = ctx.validate();
        for diagnostic in &diagnostics {
            println!("{}", diagnostic);
        }
        assert!(diagnostics.is_empty());

        let person_obj = ctx.db.find_input_object_by_name("Person".to_string());

        if let Some(person) = person_obj {
            let field_ty_directive: Vec<String> = person
                .input_fields_definition()
                .iter()
                .filter_map(|f| {
                    if let Some(field_ty) = f.ty().ty(&ctx.db) {
                        match field_ty.as_ref() {
                            Definition::ScalarTypeDefinition(scalar) => {
                                let dir_names: Vec<String> = scalar
                                    .directives()
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

        let ctx = ApolloCompiler::new(input);
        let diagnostics = ctx.validate();
        for diagnostic in &diagnostics {
            println!("{}", diagnostic);
        }
        // the scalar warning diagnostic
        assert_eq!(diagnostics.len(), 1);

        let object_types = ctx.object_types();
        let object_names: Vec<_> = object_types.iter().map(|op| op.name()).collect();
        assert_eq!(
            ["Mutation", "Product", "Query", "Review", "User"],
            object_names.as_slice()
        );
    }
}
