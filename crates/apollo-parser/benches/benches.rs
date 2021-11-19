use apollo_parser::ast;
use criterion::*;

fn parse_query(query: &str) {
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
}

fn bench_parser_peek_n(c: &mut Criterion) {
    let query = "query ExampleQuery($topProductsFirst: Int) {\n  me { \n    id\n  }\n  topProducts(first:  $topProductsFirst) {\n    name\n    price\n    inStock\n weight\n test test test test test test test test test test test test }\n}";

    c.bench_function("parser_peek_n", move |b| b.iter(|| parse_query(query)));
}

criterion_group!(benches, bench_parser_peek_n);
criterion_main!(benches);
