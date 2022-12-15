#![doc = include_str!("../README.md")]

pub mod database;
pub mod diagnostics;
#[cfg(test)]
mod tests;
pub mod validation;

use std::path::Path;

use salsa::ParallelDatabase;
use validation::ValidationDatabase;

pub use database::{
    hir, AstDatabase, DocumentDatabase, FileId, HirDatabase, InputDatabase, RootDatabase, Source,
};
pub use diagnostics::ApolloDiagnostic;

pub struct ApolloCompiler {
    pub db: RootDatabase,
    next_file_id: FileId,
}

/// Apollo compiler creates a context around your GraphQL. It creates refernces
/// between various GraphQL types in scope.
///
/// ## Example
///
/// ```rust
/// use apollo_compiler::ApolloCompiler;
///
/// let input = r#"
///   interface Pet {
///     name: String
///   }
///
///   type Dog implements Pet {
///     name: String
///     nickname: String
///     barkVolume: Int
///   }
///
///   type Cat implements Pet {
///     name: String
///     nickname: String
///     meowVolume: Int
///   }
///
///   union CatOrDog = Cat | Dog
///
///   type Human {
///     name: String
///     pets: [Pet]
///   }
///
///   type Query {
///     human: Human
///   }
/// "#;
///
/// let mut compiler = ApolloCompiler::new();
/// compiler.create_schema(input, "schema.graphql");
///
/// let diagnostics = compiler.validate();
/// for diagnostic in &diagnostics {
///     // this will pretty-print diagnostics using the miette crate.
///     println!("{}", diagnostic);
/// }
/// assert!(diagnostics.is_empty());
/// ```
impl ApolloCompiler {
    /// Create a new instance of Apollo Compiler.
    pub fn new() -> Self {
        Default::default()
    }

    /// Create a new instance of Apollo Compiler,
    /// and configure the parser with the given recursion limit.
    pub fn with_recursion_limit(limit: usize) -> Self {
        let mut compiler = Self::new();
        compiler.db.set_recursion_limit(Some(limit));
        compiler
    }

    fn add_input(&mut self, source: Source) -> FileId {
        let next_file_id = FileId(self.next_file_id.0 + 1);
        let file_id = std::mem::replace(&mut self.next_file_id, next_file_id);

        let mut sources = self.db.source_files();
        sources.push(file_id);
        self.db.set_input(file_id, source);
        self.db.set_source_files(sources);

        file_id
    }

    /// Add a document with executable _and_ type system definitions and
    /// extensions to the compiler.
    ///
    /// The `path` argument is used to display diagnostics. If your GraphQL document
    /// doesn't come from a file, you can make up a name or provide the empty string.
    /// It does not need to be unique.
    ///
    /// Returns a `FileId` that you can use to update the source text of this document.
    pub fn create_document(&mut self, input: &str, path: impl AsRef<Path>) -> FileId {
        let filename = path.as_ref().to_owned();
        self.add_input(Source::document(filename, input))
    }

    /// Add a schema - a document with type system definitions and extensions only
    /// - to the compiler.
    ///
    /// The `path` argument is used to display diagnostics. If your GraphQL document
    /// doesn't come from a file, you can make up a name or provide the empty string.
    /// It does not need to be unique.
    ///
    /// Returns a `FileId` that you can use to update the source text of this document.
    pub fn create_schema(&mut self, input: &str, path: impl AsRef<Path>) -> FileId {
        let filename = path.as_ref().to_owned();
        self.add_input(Source::schema(filename, input))
    }

    /// Add a an executable document to the compiler.
    ///
    /// The `path` argument is used to display diagnostics. If your GraphQL document
    /// doesn't come from a file, you can make up a name or provide the empty string.
    /// It does not need to be unique.
    ///
    /// Returns a `FileId` that you can use to update the source text of this document.
    pub fn create_executable(&mut self, input: &str, path: impl AsRef<Path>) -> FileId {
        let filename = path.as_ref().to_owned();
        self.add_input(Source::executable(filename, input))
    }

    /// Update an existing GraphQL document with new source text. Queries that depend
    /// on this document will be recomputed.
    pub fn update_document(&mut self, file_id: FileId, input: &str) {
        let document = self.db.input(file_id);
        self.db.set_input(
            file_id,
            Source::document(document.filename().to_owned(), input),
        )
    }

    /// Update an existing GraphQL document with new source text. Queries that depend
    /// on this document will be recomputed.
    pub fn update_schema(&mut self, file_id: FileId, input: &str) {
        let schema = self.db.input(file_id);
        self.db
            .set_input(file_id, Source::schema(schema.filename().to_owned(), input))
    }

    /// Update an existing GraphQL document with new source text. Queries that depend
    /// on this document will be recomputed.
    pub fn update_executable(&mut self, file_id: FileId, input: &str) {
        let executable = self.db.input(file_id);
        self.db.set_input(
            file_id,
            Source::executable(executable.filename().to_owned(), input),
        )
    }

    /// Get a snapshot of the current database.
    pub fn snapshot(&self) -> salsa::Snapshot<RootDatabase> {
        self.db.snapshot()
    }

    /// Validate your GraphQL input. Returns Diagnostics that you can pretty-print.
    ///
    /// ## Example
    /// ```rust
    /// use apollo_compiler::ApolloCompiler;
    /// let input = r#"
    /// type Query {
    ///   website: URL,
    ///   amount: Int
    /// }
    /// "#;
    ///
    /// let mut compiler = ApolloCompiler::new();
    /// compiler.create_document(input, "document.graphql");
    ///
    /// let diagnostics = compiler.validate();
    /// for diagnostic in &diagnostics {
    ///     println!("{}", diagnostic);
    /// }
    /// assert_eq!(diagnostics.len(), 1);
    /// ```
    pub fn validate(&self) -> Vec<ApolloDiagnostic> {
        self.db.validate()
    }
}

impl Default for ApolloCompiler {
    fn default() -> Self {
        let mut db = RootDatabase::default();
        // TODO(@goto-bus-stop) can we make salsa fill in these defaults for us…?
        db.set_recursion_limit(None);
        db.set_source_files(vec![]);

        Self {
            db,
            next_file_id: FileId(0),
        }
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use crate::{hir::Definition, ApolloCompiler, DocumentDatabase, HirDatabase};

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
        compiler.create_document(schema, "schema.graphql");
        compiler.create_executable(query, "query.graphql");
    }

    #[test]
    fn it_accesses_operation_definition_parts() {
        let input = r#"
query ExampleQuery($definedVariable: Int, $definedVariable2: Int) {
  topProducts(first: $definedVariable) {
    type
  }
  ... vipCustomer
}

fragment vipCustomer on User {
  id
  name
  profilePic(size: $definedVariable2)
}

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

        let mut compiler = ApolloCompiler::new();
        let document_id = compiler.create_document(input, "document.graphql");

        let diagnostics = compiler.validate();
        for diagnostic in &diagnostics {
            println!("{}", diagnostic);
        }
        assert!(diagnostics.is_empty());

        let operations = compiler.db.operations(document_id);
        let operation_names: Vec<_> = operations.iter().filter_map(|op| op.name()).collect();
        assert_eq!(["ExampleQuery"], operation_names.as_slice());

        let fragments = compiler.db.fragments(document_id);
        let fragment_names: Vec<_> = fragments.iter().map(|fragment| fragment.name()).collect();
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
        let document_id = compiler.create_document(input, "document.graphql");

        let diagnostics = compiler.validate();
        for diagnostic in &diagnostics {
            println!("{}", diagnostic);
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
        let document_id = compiler.create_document(input, "document.graphql");

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
"#;

        let mut compiler = ApolloCompiler::new();
        let document_id = compiler.create_document(input, "document.graphql");

        let diagnostics = compiler.validate();
        for diagnostic in &diagnostics {
            println!("{}", diagnostic);
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
        let customer_query = r#"{ customer }"#;
        let colliding_query = r#"query getProduct { topProducts { type, price } }"#;

        let mut compiler = ApolloCompiler::new();
        compiler.create_schema(schema, "schema.graphql");
        compiler.create_executable(product_query, "query.graphql");
        compiler.create_executable(customer_query, "query.graphql");
        compiler.create_executable(colliding_query, "query.graphql");

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
  ... vipCustomer
}

fragment vipCustomer on User {
  id
  name
  profilePic(size: 50)
}
"#;

        let mut compiler = ApolloCompiler::new();
        compiler.create_schema(schema, "schema.graphql");
        let query_id = compiler.create_executable(query, "query.graphql");

        let diagnostics = compiler.validate();
        for diagnostic in &diagnostics {
            println!("{}", diagnostic);
        }
        assert!(diagnostics.is_empty());

        let op = compiler
            .db
            .find_operation_by_name(query_id, String::from("getProduct"));
        let fragment_in_op: Vec<crate::hir::FragmentDefinition> = op
            .unwrap()
            .selection_set()
            .selection()
            .iter()
            .filter_map(|sel| match sel {
                crate::hir::Selection::FragmentSpread(frag) => {
                    Some(frag.fragment(&compiler.db)?.as_ref().clone())
                }
                _ => None,
            })
            .collect();
        let fragment_fields: Vec<crate::hir::Field> = fragment_in_op
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
        compiler.create_document(input, "document.graphql");

        let diagnostics = compiler.validate();
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

        let mut compiler = ApolloCompiler::new();
        compiler.create_document(input, "document.graphql");

        let diagnostics = compiler.validate();
        for diagnostic in &diagnostics {
            println!("{}", diagnostic);
        }
        assert!(diagnostics.is_empty());

        let scalars = compiler.db.scalars();

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

        let mut compiler = ApolloCompiler::new();
        compiler.create_document(input, "document.graphql");

        let diagnostics = compiler.validate();
        for diagnostic in &diagnostics {
            println!("{}", diagnostic);
        }
        assert!(diagnostics.is_empty());

        let enums = compiler.db.enums();
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

        let mut compiler = ApolloCompiler::new();
        compiler.create_document(input, "document.graphql");

        let diagnostics = compiler.validate();
        for diagnostic in &diagnostics {
            println!("{}", diagnostic);
        }
        assert!(diagnostics.is_empty());

        let unions = compiler.db.unions();
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
            .object(&compiler.db);

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

        let mut compiler = ApolloCompiler::new();
        compiler.create_document(input, "document.graphql");

        let diagnostics = compiler.validate();
        for diagnostic in &diagnostics {
            println!("{}", diagnostic);
        }
        assert!(diagnostics.is_empty());

        let directives = compiler.db.directive_definitions();
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

        let mut compiler = ApolloCompiler::new();
        compiler.create_document(input, "document.graphql");

        let diagnostics = compiler.validate();
        for diagnostic in &diagnostics {
            println!("{}", diagnostic);
        }
        assert!(diagnostics.is_empty());

        let input_objects = compiler.db.input_objects();
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
    fn it_accesses_object_directive_name() {
        let input = r#"

type Book @directiveA(name: "pageCount") @directiveB(name: "author") {
  id: ID!
}
"#;

        let mut compiler = ApolloCompiler::new();
        compiler.create_document(input, "document.graphql");

        let diagnostics = compiler.validate();
        for diagnostic in &diagnostics {
            println!("{}", diagnostic);
        }
        assert!(diagnostics.is_empty());

        let book_obj = compiler
            .db
            .find_object_type_by_name("Book".to_string())
            .unwrap();

        let directive_names: Vec<&str> = book_obj.directives().iter().map(|d| d.name()).collect();
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
        compiler.create_document(input, "document.graphql");

        let diagnostics = compiler.validate();
        for diagnostic in &diagnostics {
            println!("{}", diagnostic);
        }
        assert!(diagnostics.is_empty());

        let person_obj = compiler.db.find_object_type_by_name("Person".to_string());

        if let Some(person) = person_obj {
            let field_ty_directive: Vec<String> = person
                .fields_definition()
                .iter()
                .filter_map(|f| {
                    // get access to the actual definition the field is using
                    if let Some(field_ty) = f.ty().ty(&compiler.db) {
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
                            if let Some(input_ty) = val.ty().ty(&compiler.db) {
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

        let mut compiler = ApolloCompiler::new();
        compiler.create_document(input, "document.graphql");

        let diagnostics = compiler.validate();
        for diagnostic in &diagnostics {
            println!("{}", diagnostic);
        }
        assert!(diagnostics.is_empty());

        let person_obj = compiler.db.find_input_object_by_name("Person".to_string());

        if let Some(person) = person_obj {
            let field_ty_directive: Vec<String> = person
                .input_fields_definition()
                .iter()
                .filter_map(|f| {
                    if let Some(field_ty) = f.ty().ty(&compiler.db) {
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

        let mut compiler = ApolloCompiler::new();
        compiler.create_document(input, "document.graphql");

        let diagnostics = compiler.validate();
        for diagnostic in &diagnostics {
            println!("{}", diagnostic);
        }
        // the scalar warning diagnostic
        assert_eq!(diagnostics.len(), 1);

        let object_types = compiler.db.object_types();
        let object_names: Vec<_> = object_types.iter().map(|op| op.name()).collect();
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
        compiler.create_document(input, "document.graphql");

        let diagnostics = compiler.validate();
        for diagnostic in &diagnostics {
            println!("{}", diagnostic);
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
        let input_id = compiler.create_document(input, "document.graphql");

        let object_type = compiler
            .db
            .find_object_type_by_name("Query".into())
            .unwrap();
        assert!(object_type.directives().is_empty());

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
        assert_eq!(object_type.directives().len(), 1);
    }
}
