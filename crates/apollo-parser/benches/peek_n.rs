#![feature(test)]
extern crate test;
use apollo_parser::ast;
use test::{black_box, Bencher};

#[bench]
fn bench_peek_n(b: &mut Bencher) {
    let query = "query ExampleQuery($topProductsFirst: Int) {\n  me { \n    id\n  }\n  topProducts(first:  $topProductsFirst) {\n    name\n    price\n    inStock\n weight\n test test test test test test test test test test test test }\n}";

    b.iter(|| {
        let parser = apollo_parser::Parser::new(query);
        let tree = parser.parse();

        if !tree.errors().is_empty() {
            panic!("error parsing query: {:?}", tree.errors());
        }
        let document = tree.document();

        for definition in document.definitions() {
            if let ast::Definition::OperationDefinition(operation) = definition {
                let selection_set = operation
                    .selection_set()
                    .expect("the node SelectionSet is not optional in the spec; qed");
                for selection in selection_set.selections() {
                    match selection {
                        ast::Selection::Field(field) => {
                            let _selection_set = field.selection_set();
                        }
                        _ => {}
                    }
                }
            }
        }
    });
}
