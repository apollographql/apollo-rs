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
            std::hint::black_box(doc);
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
            std::hint::black_box(doc);
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
            _ = std::hint::black_box(doc);
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
            std::hint::black_box(doc);
        });
    });
}

fn bench_many_extensions(c: &mut Criterion) {
    let num_extensions = 10_000;
    let mut schema = String::new();
    schema.push_str("type Query { a: A }\n");
    for i in 1..=num_extensions {
        schema.push_str(&format!("interface I{i} {{ f{i}: String }}\n"));
    }
    schema.push_str("type A { f0: String }\n");
    for i in 1..=num_extensions {
        schema.push_str(&format!(
            "extend type A implements I{i} {{ f{i}: String }}\n"
        ));
    }
    let schema = Schema::parse_and_validate(&schema, "schema.graphql").unwrap();

    let mut query = String::new();
    query.push_str("{ a { ");
    for i in 1..=num_extensions {
        query.push_str(&format!("f{i} "));
    }
    query.push_str("} }");

    c.bench_function("many_extensions", move |b| {
        b.iter(|| {
            let doc =
                ExecutableDocument::parse_and_validate(&schema, &query, "query.graphql").unwrap();
            std::hint::black_box(doc);
        });
    });
}

criterion_group!(
    fields,
    bench_many_same_field,
    bench_many_same_nested_field,
    bench_many_arguments,
    bench_many_types,
    bench_many_extensions,
);
criterion_main!(fields);
