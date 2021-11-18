use apollo_parser::{ast, Lexer};
use criterion::*;

fn parse_schema(schema: &str) {
    let parser = apollo_parser::Parser::new(schema);
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

fn bench_supergraph_parser(c: &mut Criterion) {
    let schema = include_str!("../test_data/parser/ok/0032_supergraph.graphql");

    c.bench_function("supergraph_parser", move |b| {
        b.iter(|| parse_schema(schema))
    });
}

fn bench_supergraph_lexer(c: &mut Criterion) {
    let schema = include_str!("../test_data/parser/ok/0032_supergraph.graphql");

    c.bench_function("supergraph_lexer", move |b| {
        b.iter(|| {
            let _ = Lexer::new(schema);
        })
    });
}

criterion_group!(benches, bench_supergraph_lexer, bench_supergraph_parser);
criterion_main!(benches);
