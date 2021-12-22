mod database;
mod passes;

use std::{collections::HashSet, sync::Arc};

pub use database::{Database, SourceDatabase};

use apollo_parser::ast::{self, AstNode};
use miette::{Diagnostic, NamedSource, Report, SourceSpan};
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

pub fn validate(src: &str) {
    let mut db = Database::default();

    db.set_input_string((), Arc::new(src.to_string()));
    let doc = db.parse();

    // println!("Now, the length is {}.", db.length(()));
    passes::unused_variables::check(&doc);
    let (implements_interfaces, defined_interfaces) =
        passes::unused_implements_interfaces::check(&doc);
    if !implements_interfaces.is_empty() {
        let undefined_interfaces: HashSet<ast::Name> = implements_interfaces
            .difference(&defined_interfaces)
            .cloned()
            .collect();
        for interface in undefined_interfaces {
            let syntax = interface.syntax();
            let index: usize = syntax.text_range().start().into();
            let len: usize = syntax.text().len().into();

            let err = Report::new(GraphQLUndefinedInterfacesError {
                src: NamedSource::new("schema.graphql", src.to_owned()),
                span: (index, len).into(),
                message: "This interface is not defined.".to_string(),
                ty: interface.text().to_string(),
            });

            println!("{:?}", err);
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

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
query ExampleQuery() {
  topProducts(first: $undefinedVariable) {
    name
  }
}"#;

        validate(input)
    }
}
