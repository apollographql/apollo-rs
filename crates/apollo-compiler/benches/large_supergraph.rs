use apollo_compiler::ast::Document;
use apollo_compiler::ExecutableDocument;
use apollo_compiler::Schema;
use criterion::*;

// Path to a large production supergraph schema. Points at a local corpus
// file that is not committed to the repo.
const SCHEMA_PATH: &str = "TBD: path to a large supergraph schema";

// Directory of `.graphql` operation files to validate against the schema.
const OPS_DIR: &str = "TBD: path to an operations corpus";

fn load(path: &str) -> String {
    std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{path}: {e}"))
}

fn bench_large_supergraph(c: &mut Criterion) {
    let schema_src = load(SCHEMA_PATH);

    let mut group = c.benchmark_group("large_supergraph");
    group.sample_size(20);
    group.measurement_time(std::time::Duration::from_secs(30));

    group.bench_function("parse_schema_ast", |b| {
        b.iter(|| {
            let doc = Document::parse(&schema_src, "supergraph.graphql").unwrap();
            std::hint::black_box(doc);
        })
    });

    group.bench_function("parse_schema", |b| {
        b.iter(|| {
            let schema = Schema::parse(&schema_src, "supergraph.graphql").unwrap();
            std::hint::black_box(schema);
        })
    });

    group.bench_function("clone_schema", |b| {
        let schema = Schema::parse(&schema_src, "supergraph.graphql").unwrap();
        b.iter(|| {
            std::hint::black_box(schema.clone());
        })
    });

    group.bench_function("parse_and_validate_schema", |b| {
        b.iter(|| {
            let _ = std::hint::black_box(Schema::parse_and_validate(
                &schema_src,
                "supergraph.graphql",
            ));
        })
    });

    // Server-shaped workload: validate a corpus of real operations against
    // the fixed schema. Registered last: freezing is process-global, so it
    // must not affect the schema-build benches above.
    group.bench_function("parse_and_validate_operations", |b| {
        // This supergraph is rejected by the September 2025 rule against
        // `@deprecated` on implementing fields; skip schema validation so
        // the corpus is usable while it is still being fixed.
        let schema = apollo_compiler::validation::Valid::assume_valid(
            Schema::parse(&schema_src, "supergraph.graphql").unwrap(),
        );
        // Freeze after schema build, before parsing untrusted documents.
        apollo_compiler::freeze_interning();
        let mut ops: Vec<(std::path::PathBuf, String)> = std::fs::read_dir(OPS_DIR)
            .unwrap_or_else(|e| panic!("{OPS_DIR}: {e}"))
            .map(|entry| {
                let path = entry.unwrap().path();
                let src = load(path.to_str().expect("UTF-8 path"));
                (path, src)
            })
            .filter(|(path, _)| path.extension().is_some_and(|ext| ext == "graphql"))
            .collect();
        ops.sort_unstable();
        b.iter(|| {
            for (path, src) in &ops {
                let result = ExecutableDocument::parse_and_validate(&schema, src, path);
                let _ = std::hint::black_box(result);
            }
        })
    });

    group.finish();
}

criterion_group!(benches, bench_large_supergraph);
criterion_main!(benches);
