use apollo_compiler::ExecutableDocument;
use apollo_compiler::Schema;
use criterion::*;

fn compile(schema: &str, query: &str) -> (Schema, ExecutableDocument) {
    let schema = Schema::parse(schema, "schema.graphql");
    let doc = ExecutableDocument::parse(&schema, query, "query.graphql");
    black_box((schema, doc))
}

fn compile_and_validate(schema: &str, query: &str) {
    let schema = Schema::parse(schema, "schema.graphql");
    let doc = ExecutableDocument::parse(&schema, query, "query.graphql");
    let _ = black_box(schema.validate(Default::default()));
    let _ = black_box(doc.validate(&schema, Default::default()));
}

fn bench_simple_query_compiler(c: &mut Criterion) {
    let query = include_str!("testdata/simple_query.graphql");
    let schema = include_str!("testdata/simple_schema.graphql");

    c.bench_function("simple_query_compiler", move |b| {
        b.iter(|| compile(schema, query))
    });
}

fn bench_simple_query_compiler_with_validation(c: &mut Criterion) {
    let query = include_str!("testdata/simple_query.graphql");
    let schema = include_str!("testdata/simple_schema.graphql");

    c.bench_function("simple_query_compiler_with_validation", move |b| {
        b.iter(|| compile_and_validate(schema, query))
    });
}

fn bench_compiler_supergraph(c: &mut Criterion) {
    let schema = include_str!("testdata/supergraph.graphql");
    let query = include_str!("testdata/supergraph_query.graphql");

    c.bench_function("supergraph_compiler", move |b| {
        b.iter(|| compile(schema, query))
    });
}
fn bench_compiler_supergraph_with_validation(c: &mut Criterion) {
    let schema = include_str!("testdata/supergraph.graphql");
    let query = include_str!("testdata/supergraph_query.graphql");

    c.bench_function("supergraph_compiler_with_validation", move |b| {
        b.iter(|| compile_and_validate(schema, query))
    });
}

criterion_group!(
    benches,
    bench_compiler_supergraph,
    bench_compiler_supergraph_with_validation,
    bench_simple_query_compiler,
    bench_simple_query_compiler_with_validation
);
criterion_main!(benches);
