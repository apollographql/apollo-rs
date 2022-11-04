use apollo_parser::{ast, Lexer};
use criterion::*;

fn parse_schema(schema: &str) {
    let parser = apollo_parser::Parser::new(schema);
    let tree = parser.parse();
    let errors = tree.errors().collect::<Vec<_>>();

    if !errors.is_empty() {
        panic!("error parsing query: {:?}", errors);
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
            let lexer = Lexer::new(schema);

            for token_res in lexer {
                let _ = token_res;
            }
        })
    });
}

criterion_group!(benches, bench_supergraph_lexer, bench_supergraph_parser);
criterion_main!(benches);
