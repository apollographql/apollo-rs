mod database;
mod passes;

use std::sync::Arc;

pub use database::{Database, DatabaseTrait};

use apollo_parser::SyntaxTree;

pub fn check(ast: SyntaxTree) {
    let mut db = Database::default();

    db.set_input_string((), Arc::new("Hello, world".to_string()));

    println!("Now, the length is {}.", db.length(()));
    passes::unused_variables::check(ast)
}

#[cfg(test)]
mod test {
    use super::*;

    use apollo_parser::Parser;

    #[test]
    fn it_validates_undefined_variable_in_query() {
        let input = r#"
query ExampleQuery() {
  topProducts(first: $undefinedVariable) {
    name
  }
}"#;
        let parser = Parser::new(input);
        let ast = parser.parse();

        assert_eq!(ast.errors().len(), 0);

        check(ast)
    }
}
