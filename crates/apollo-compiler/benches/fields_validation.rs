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

fn bench_many_arguments(c: &mut Criterion) {
    let schema =
        Schema::parse_and_validate("type Query { hello: String! }", "schema.graphql").unwrap();
    let args = (0..2_000).fold(String::new(), |mut acc, i| {
        use std::fmt::Write;
        _ = writeln!(&mut acc, "arg{i}: {i}");
        acc
    });
    let field = format!("hello({args})");
    let query = format!("{{ {field} {field} }}");

    c.bench_function("many_arguments", move |b| {
        b.iter(|| {
            // Will return errors but that's cool
            let doc = ExecutableDocument::parse_and_validate(&schema, &query, "query.graphql");
            _ = black_box(doc);
        });
    });
}

fn bench_many_types(c: &mut Criterion) {
    let schema = Schema::parse_and_validate(
        "
        interface Abstract {
          field: Abstract
          leaf: Int
        }
        interface Abstract1 {
          field: Abstract
          leaf: Int
        }
        interface Abstract2 {
          field: Abstract
          leaf: Int
        }
        type Concrete1 implements Abstract & Abstract1 {
          field: Abstract
          leaf: Int
        }
        type Concrete2 implements Abstract & Abstract2 {
          field: Abstract
          leaf: Int
        }
        type Query {
            field: Abstract
        }
    ",
        "schema.graphql",
    )
    .unwrap();
    let query = format!(
        "
        fragment multiply on Abstract {{
           field {{
             ... on Abstract1 {{ field {{ leaf }} }}
             ... on Abstract2 {{ field {{ leaf }} }}
             ... on Concrete1 {{ field {{ leaf }} }}
             ... on Concrete2 {{ field {{ leaf }} }}
           }}
        }}

        query DeepAbstractConcrete {{
            {open}{close}
        }}
    ",
        open = "field { ...multiply ".repeat(100),
        close = "}".repeat(100)
    );

    c.bench_function("many_types", move |b| {
        b.iter(|| {
            let doc =
                ExecutableDocument::parse_and_validate(&schema, &query, "query.graphql").unwrap();
            black_box(doc);
        });
    });
}

criterion_group!(
    fields,
    bench_many_same_field,
    bench_many_same_nested_field,
    bench_many_arguments,
    bench_many_types,
);
criterion_main!(fields);
