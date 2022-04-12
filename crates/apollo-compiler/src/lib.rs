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
}
