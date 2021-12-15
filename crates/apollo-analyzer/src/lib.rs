mod passes;

use apollo_parser::SyntaxTree;

pub fn check(ast: SyntaxTree) {
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
