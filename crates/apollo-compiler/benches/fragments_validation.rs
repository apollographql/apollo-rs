use apollo_compiler::ExecutableDocument;
use apollo_compiler::Schema;
use criterion::*;
use std::fmt::Write;

fn bench_big_schema_many_fragments(c: &mut Criterion) {
    const NUM_INTERFACES: usize = 200;
    const NUM_OBJECTS: usize = 10_000;

    let mut sdl = String::new();
    for i in 0..NUM_INTERFACES {
        _ = writeln!(&mut sdl, r#"interface Intf{i} {{ field: Int! }}"#);
    }
    for o in 0..NUM_OBJECTS {
        let i = o % NUM_INTERFACES;
        _ = writeln!(
            &mut sdl,
            r#"type Ty{o} implements Intf{i} {{ field: Int! }}"#
        );
    }

    _ = writeln!(&mut sdl, "type Query {{");
    for i in 0..NUM_INTERFACES {
        _ = writeln!(&mut sdl, "  intf{i}: Intf{i}");
    }
    _ = writeln!(&mut sdl, "}}");

    let schema = Schema::parse_and_validate(sdl, "schema.graphql").unwrap();
    let mut selection = String::new();
    let mut fragments = String::new();
    for i in 0..NUM_INTERFACES {
        _ = writeln!(&mut selection, "  intf{i} {{ ...frag{i} }}");
        _ = writeln!(&mut fragments, "fragment frag{i} on Ty{i} {{ field }}");
    }
    let query = format!(
        "query {{
  {selection}}}
{fragments}"
    );

    c.bench_function("big_schema_many_fragments", move |b| {
        b.iter(|| {
            let doc =
                ExecutableDocument::parse_and_validate(&schema, &query, "query.graphql").unwrap();
            black_box(doc);
        });
    });
}

criterion_group!(fragments, bench_big_schema_many_fragments,);
criterion_main!(fragments);
