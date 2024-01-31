use apollo_compiler::ExecutableDocument;
use apollo_compiler::Schema;
use criterion::*;

fn bench_many_same_field(c: &mut Criterion) {
    let schema =
        Schema::parse_and_validate("type Query { hello: String! }", "schema.graphql").unwrap();
    let query = format!("{{ {} }}", "hello ".repeat(1_000));

    c.bench_function("many_same_field", move |b| {
        b.iter(|| {
            let doc =
                ExecutableDocument::parse_and_validate(&schema, &query, "query.graphql").unwrap();
            black_box(doc);
        });
    });
}

fn bench_many_same_nested_field(c: &mut Criterion) {
    let schema = Schema::parse_and_validate(
        "
        type Nested { hello: String! }
        type Query { nested: Nested! }
    ",
        "schema.graphql",
    )
    .unwrap();
    let query = format!("{{ {} }}", "nested { hello } ".repeat(1_000));

    c.bench_function("many_same_nested_field", move |b| {
        b.iter(|| {
            let doc =
                ExecutableDocument::parse_and_validate(&schema, &query, "query.graphql").unwrap();
            black_box(doc);
        });
    });
}

criterion_group!(benches, bench_many_same_field, bench_many_same_nested_field);
criterion_main!(benches);
