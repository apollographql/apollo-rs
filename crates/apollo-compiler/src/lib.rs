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

    pub fn operations(&self) -> values::Operations {
        self.db.operations()
    }

    pub fn fragments(&self) -> values::Fragments {
        self.db.fragments()
    }

    pub fn schema(&self) -> Arc<values::SchemaDefinition> {
        self.db.schema()
    }

    pub fn object_types(&self) -> Arc<Vec<values::ObjectTypeDefinition>> {
        self.db.object_types()
    }

    pub fn scalars(&self) -> Arc<Vec<values::ScalarDefinition>> {
        self.db.scalars()
    }

    pub fn enums(&self) -> Arc<Vec<values::EnumDefinition>> {
        self.db.enums()
    }

    pub fn unions(&self) -> Arc<Vec<values::UnionDefinition>> {
        self.db.unions()
    }

    pub fn directive_definitions(&self) -> Arc<Vec<values::DirectiveDefinition>> {
        self.db.directive_definitions()
    }

    pub fn input_objects(&self) -> Arc<Vec<values::InputObjectDefinition>> {
        self.db.input_objects()
    }
}

#[cfg(test)]
mod test {
    use crate::ApolloCompiler;

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
    topProducts: Products
}
"#;

        let ctx = ApolloCompiler::new(input);
        let errors = ctx.validate();

        assert!(errors.is_empty());

        let operations = ctx.operations();
        let operation_names: Vec<_> = operations.iter().filter_map(|op| op.name()).collect();
        assert_eq!(["ExampleQuery"], operation_names.as_slice());

        let fragments = ctx.fragments();
        let fragment_names: Vec<_> = fragments.iter().map(|fragment| fragment.name()).collect();
        assert_eq!(["vipCustomer"], fragment_names.as_slice());

        let operation_variables: Vec<String> = match operations.find("ExampleQuery") {
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
        let errors = ctx.validate();

        assert!(errors.is_empty());

        let operations = ctx.operations();
        let fields = operations.find("ExampleQuery").unwrap().fields(&ctx.db);
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
        let errors = ctx.validate();

        assert!(errors.is_empty());
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
        let errors = ctx.validate();

        assert!(errors.is_empty());

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
        let errors = ctx.validate();

        assert!(errors.is_empty());

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
        let errors = ctx.validate();

        assert!(errors.is_empty());

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
        let errors = ctx.validate();

        assert!(errors.is_empty());

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
  point1: Point2D
  point2: Point2D
}

input Point2D {
  x: Float
  y: Float
}
"#;

        let ctx = ApolloCompiler::new(input);
        let errors = ctx.validate();

        assert!(errors.is_empty());

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
}
