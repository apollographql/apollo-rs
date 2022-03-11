mod diagnostics;
mod queries;
mod validator;
mod values;

use std::sync::Arc;

use apollo_parser::{ast, SyntaxTree};
pub use queries::database::{Database, SourceDatabase};

use diagnostics::ApolloDiagnostic;
use validator::Validator;

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

    pub fn validate(&self) -> Vec<ApolloDiagnostic> {
        let validator = Validator::new(&self.db);
        validator.validate()
    }

    pub fn syntax_errors(&self) -> Arc<Vec<ApolloDiagnostic>> {
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
            ctx.operations().iter().filter_map(|op| op.name()).collect();
        assert_eq!(["ExampleQuery"], operation_names.as_slice());
        let fragments: Vec<String> = ctx
            .fragments()
            .iter()
            .map(|fragment| fragment.name())
            .collect();
        assert_eq!(["vipCustomer"], fragments.as_slice());

        let operation_variables: Vec<String> = ctx
            .operations()
            .find("ExampleQuery")
            .unwrap()
            .variables()
            .unwrap()
            .iter()
            .map(|var| var.name.clone())
            .collect();
        assert_eq!(["definedVariable"], operation_variables.as_slice());
        // let operation_variables = ctx.operations().find("ExampleQuery").variables().find("definedVariable").ty();
        let fragment_fields = ctx.fragments().find("vipCustomer").unwrap();
        dbg!(fragment_fields);
    }
}
