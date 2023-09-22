use apollo_compiler::{ApolloCompiler, HirDatabase};
use criterion::*;

fn compile(schema: &str, query: &str) -> ApolloCompiler {
    let mut compiler = ApolloCompiler::new();
    compiler.add_type_system(schema, "schema.graphql");
    let executable_id = compiler.add_executable(query, "query.graphql");

    black_box(compiler.db.operations(executable_id));
    black_box(compiler.db.object_types());

    compiler
}

fn compile_and_validate(schema: &str, query: &str) {
    let mut compiler = ApolloCompiler::new();
    compiler.add_type_system(schema, "schema.graphql");
    let executable_id = compiler.add_executable(query, "query.graphql");

    black_box(compiler.validate());
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
