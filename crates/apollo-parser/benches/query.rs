use apollo_parser::{ast, Lexer};
use criterion::*;

fn parse_query(query: &str) {
    let parser = apollo_parser::Parser::new(query);
    let tree = parser.parse();

    if tree.errors().len() != 0 {
        panic!("error parsing query: {:?}", tree.errors());
    }
    let document = tree.document();

    for definition in document.definitions() {
        if let ast::Definition::OperationDefinition(operation) = definition {
            let selection_set = operation
                .selection_set()
                .expect("the node SelectionSet is not optional in the spec; qed");
            for selection in selection_set.selections() {
                if let ast::Selection::Field(field) = selection {
                    let _selection_set = field.selection_set();
                }
            }
        }
    }
}

fn bench_query_parser(c: &mut Criterion) {
    let query = "query ExampleQuery($topProductsFirst: Int) {\n  me { \n    id\n  }\n  topProducts(first:  $topProductsFirst) {\n    name\n    price\n    inStock\n weight\n test test test test test test test test test test test test }\n}";

    c.bench_function("query_parser", move |b| b.iter(|| parse_query(query)));
}

fn bench_query_lexer(c: &mut Criterion) {
    let query = "query ExampleQuery($topProductsFirst: Int) {\n  me { \n    id\n  }\n  topProducts(first:  $topProductsFirst) {\n    name\n    price\n    inStock\n weight\n test test test test test test test test test test test test }\n}";

    c.bench_function("query_lexer", move |b| {
        b.iter(|| {
            let lexer = Lexer::new(query);

            for token_res in lexer {
                let _ = token_res;
            }
        })
    });
}

fn bench_parser_many_aliases(c: &mut Criterion) {
    let query = include_str!("testdata/alias.graphql");

    c.bench_function("many_aliases", move |b| b.iter(|| parse_query(query)));
}

criterion_group!(benches, bench_parser_many_aliases, bench_query_lexer, bench_query_parser);
criterion_main!(benches);
