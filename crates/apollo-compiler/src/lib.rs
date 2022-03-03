mod passes;
mod queries;
mod values;

use std::sync::Arc;

use apollo_parser::{ast, SyntaxTree};
use queries::database::FragmentsQuery;
pub use queries::database::{Database, SourceDatabase};

use miette::{Diagnostic, NamedSource, SourceSpan};
use thiserror::Error;

#[derive(Error, Debug, Diagnostic)]
#[error("cannot find `{}` interface in this scope", self.ty)]
#[diagnostic(code("apollo-parser: semantic error"))]
struct GraphQLUndefinedInterfacesError {
    ty: String,
    #[source_code]
    src: NamedSource,
    message: String,
    #[label("{}", self.message)]
    span: SourceSpan,
}

#[derive(Error, Debug, Diagnostic)]
#[error("cannot find `{}` variable in this scope", self.ty)]
#[diagnostic(code("apollo-parser: semantic error"))]
struct GraphQLUndefinedVariablesError {
    ty: String,
    #[source_code]
    src: NamedSource,
    message: String,
    #[label("{}", self.message)]
    span: SourceSpan,
}

pub struct ApolloCompiler {
    db: Database,
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

    pub fn syntax_errors(&self) -> Arc<Vec<values::Error>> {
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
}

pub fn validate(src: &str) {
    let mut db = Database::default();

    db.set_input_string((), Arc::new(src.to_string()));

    let operations = db.operations();
    dbg!(operations);

    // println!("Now, the length is {}.", db.length(()));
    // passes::unused_variables::check(&doc);
    // let (implements_interfaces, defined_interfaces) =
    //     passes::unused_implements_interfaces::check(&doc);
    // if !implements_interfaces.is_empty() {
    //     let undefined_interfaces: HashSet<ast::Name> = implements_interfaces
    //         .difference(&defined_interfaces)
    //         .cloned()
    //         .collect();
    //     for interface in undefined_interfaces {
    //         let syntax = interface.syntax();
    //         let index: usize = syntax.text_range().start().into();
    //         let len: usize = syntax.text().len().into();
    //
    //         let err = Report::new(GraphQLUndefinedInterfacesError {
    //             src: NamedSource::new("schema.graphql", src.to_owned()),
    //             span: (index, len).into(),
    //             message: "This interface is not defined.".to_string(),
    //             ty: interface.text().to_string(),
    //         });
    //
    //         println!("{:?}", err);
    //     }
    // }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn it_creates_context() {
        let input = r#"
interface NamedEntity {
  name: String
}

interface ValuedEntity {
  value: Int
}

type Person implements NamedEntity {
  name: String
  age: Int
}

type Business implements NamedEntity & ValuedEntity {
  name: String
  value: Int
  employeeCount: Int
}"#;

        validate(input);
    }

    #[test]
    fn it_validates_undefined_interface_in_schema() {
        let input = r#"
type Person implements NamedEntity {
  name: String
  age: Int
}"#;
        validate(input)
    }

    #[test]
    fn it_validates_undefined_variable_in_query() {
        let input = r#"
query ExampleQuery($definedVariable: String) {
  topProducts(first: $undefinedVariable) {
    name
  }
}

fragment vipCustomer on User {
  id
  name
  profilePic(size: 50)
  status
}
"#;

        let ctx = ApolloCompiler::new(input);
        // let errors = ctx.validate();

        let operation_names: Vec<String> =
            ctx.operations().iter().map(|op| op.name.clone()).collect();
        assert_eq!(["ExampleQuery"], operation_names.as_slice());
        let fragments: Vec<String> = ctx
            .fragments()
            .iter()
            .map(|fragment| fragment.name.clone())
            .collect();
        assert_eq!(["vipCustomer"], fragments.as_slice());

        // let operation_variables = ctx.operations().find_one("ExampleQuery").variables().find_one("definedVariable").ty();
        // let operation_variables = ctx.operations().find_one("ExampleQuery")?.variables();
        // let fragment_fields = ctx.fragments().find_one("friendFields")?.fields();
    }
}

struct Operations {
    inner: Arc<Vec<_>>,
}

impl Operations {
    pub fn find(&self) -> Option<_>;
}

impl Deref for Operations {
    type Deref = Arc<Vec<_>>;
    fn deref(&self) -> Self::Deref {
        &self.inner
    }
}
