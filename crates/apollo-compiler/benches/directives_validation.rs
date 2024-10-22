use apollo_compiler::{ExecutableDocument, Schema};
use criterion::*;

fn bench_many_identical_directives(c: &mut Criterion) {
    let schema = format!(
        r#"
        directive @nothing repeatable on OBJECT
        type Query {} {{ hello: String! }}
        "#,
        "@nothing ".repeat(10_000),
    );

    c.bench_function("many_same_directive", move |b| {
        b.iter(|| {
            let result = Schema::parse_and_validate(&schema, "schema.graphqls").unwrap();
            black_box(result);
        });
    });
}

fn bench_many_identical_directives_query(c: &mut Criterion) {
    let schema = Schema::parse_and_validate(
        r#"
        directive @nothing repeatable on FIELD
        type Query { hello: String! }
        "#,
        "schema.graphql",
    ).unwrap();

    let query = format!(
        r#"
        {{
          hello {}
        }}
        "#,
        "@nothing ".repeat(10_000),
    );

    c.bench_function("many_same_directive_query", move |b| {
        b.iter(|| {
            let result = ExecutableDocument::parse_and_validate(&schema, &query, "query.graphql").unwrap();
            black_box(result);
        });
    });
}

fn bench_many_invalid_directives_query(c: &mut Criterion) {
    let schema = Schema::parse_and_validate(
        r#"
        type Query { hello: String! }
        "#,
        "schema.graphql",
    ).unwrap();

    let query = format!(
        r#"
        {{
          hello {}
        }}
        "#,
        "@nothing ".repeat(10_000),
    );

    c.bench_function("many_invalid_directive", move |b| {
        b.iter(|| {
            let result = ExecutableDocument::parse_and_validate(&schema, &query, "query.graphql")
                .expect_err("should have produced diagnostics");
            black_box(result);
        });
    });
}

criterion_group!(
    directives,
    bench_many_identical_directives,
    bench_many_identical_directives_query,
    bench_many_invalid_directives_query,
);
criterion_main!(directives);
