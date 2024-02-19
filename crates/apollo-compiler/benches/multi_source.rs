use apollo_compiler::ast::Document;
use apollo_compiler::ExecutableDocument;
use apollo_compiler::Schema;
use criterion::*;

fn parse_ast(schema: &str, query: &str) {
    let schema = Document::parse(schema, "schema.graphql").unwrap();
    let doc = Document::parse(query, "query.graphql").unwrap();
    black_box((schema, doc));
}

fn parse_and_validate(schema: &str, query: &str) {
    let schema = Schema::parse_and_validate(schema, "schema.graphql").unwrap();
    let doc = ExecutableDocument::parse_and_validate(&schema, query, "query.graphql").unwrap();
    black_box((schema, doc));
}

fn bench_simple_query(c: &mut Criterion) {
    let query = include_str!("testdata/simple_query.graphql");
    let schema = include_str!("testdata/simple_schema.graphql");

    c.bench_function("simple_query parse_ast", move |b| {
        b.iter(|| parse_ast(schema, query))
    });
    c.bench_function("simple_query parse_and_validate", move |b| {
        b.iter(|| parse_and_validate(schema, query))
    });
}

fn bench_supergraph(c: &mut Criterion) {
    let schema = include_str!("testdata/supergraph.graphql");
    let query = include_str!("testdata/supergraph_query.graphql");

    c.bench_function("supergraph parse_ast", move |b| {
        b.iter(|| parse_ast(schema, query))
    });
    c.bench_function("supergraph parse_and_validate", move |b| {
        b.iter(|| parse_and_validate(schema, query))
    });
}

criterion_group!(benches, bench_supergraph, bench_simple_query);
criterion_main!(benches);
