mod database;
mod passes;

use std::sync::Arc;

pub use database::{Database, DatabaseTrait};

use apollo_parser::SyntaxTree;

pub fn validate(ast: SyntaxTree) {
    let mut db = Database::default();

    db.set_input_string((), Arc::new("Hello, world".to_string()));

    println!("Now, the length is {}.", db.length(()));
    let doc = ast.document();
    passes::unused_variables::check(&doc);
    passes::unused_implements_interfaces::check(&doc);
}

#[cfg(test)]
mod test {
    use super::*;

    use apollo_parser::Parser;

    #[test]
    fn it_validates_undefined_interface_in_schema() {
        let input = r#"
type Person implements NamedEntity {
  name: String
  age: Int
}"#;
        let parser = Parser::new(input);
        let ast = parser.parse();

        assert_eq!(ast.errors().len(), 0);

        validate(ast)
    }

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

        validate(ast)
    }
}
