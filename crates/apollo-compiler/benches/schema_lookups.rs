use apollo_compiler::executable::Selection;
use apollo_compiler::executable::SelectionSet;
use apollo_compiler::schema::ExtendedType;
use apollo_compiler::ExecutableDocument;
use apollo_compiler::Name;
use apollo_compiler::Schema;
use criterion::*;
use std::fmt::Write;

/// Looks up every type, then every field of every object type, the way
/// validation and query planning query a schema.
fn bench_lookups(c: &mut Criterion, label: &str, schema: &Schema) {
    // Names as they come out of the parser.
    let names: Vec<Name> = schema.types.keys().cloned().collect();
    // Names as plain strings, like the ones taken from a request.
    let strings: Vec<String> = names.iter().map(|name| name.to_string()).collect();
    let fields: Vec<(String, String)> = schema
        .types
        .iter()
        .filter_map(|(name, ty)| match ty {
            ExtendedType::Object(object) => Some((name, object)),
            _ => None,
        })
        .flat_map(|(name, object)| {
            object
                .fields
                .keys()
                .map(move |field| (name.to_string(), field.to_string()))
        })
        .collect();

    c.bench_function(&format!("{label} types.get(Name)"), |b| {
        b.iter(|| {
            for name in &names {
                std::hint::black_box(schema.types.get(name));
            }
        })
    });
    c.bench_function(&format!("{label} types.get(&str)"), |b| {
        b.iter(|| {
            for name in &strings {
                std::hint::black_box(schema.types.get(name.as_str()));
            }
        })
    });
    c.bench_function(&format!("{label} type_field"), |b| {
        b.iter(|| {
            for (ty, field) in &fields {
                std::hint::black_box(schema.type_field(ty, field).ok());
            }
        })
    });
}

fn bench_supergraph(c: &mut Criterion) {
    let schema = Schema::parse_and_validate(
        include_str!("testdata/supergraph.graphql"),
        "supergraph.graphql",
    )
    .unwrap();
    bench_lookups(c, "supergraph", &schema);
}

fn bench_many_types(c: &mut Criterion) {
    let mut sdl = String::from("type Query { t0: T0 }\n");
    for i in 0..2_000 {
        _ = writeln!(
            &mut sdl,
            "type T{i} {{ fieldNumber{i}: String otherField: Int }}"
        );
    }
    let schema = Schema::parse_and_validate(sdl, "schema.graphql").unwrap();
    bench_lookups(c, "2000 types", &schema);
}

/// Resolves every selected field and its output type by name, the way a
/// query planner walks an operation.
fn resolve_fields(schema: &Schema, selection_set: &SelectionSet) {
    for selection in &selection_set.selections {
        let Selection::Field(field) = selection else {
            continue;
        };
        std::hint::black_box(schema.type_field(&selection_set.ty, &field.name).ok());
        std::hint::black_box(schema.types.get(field.ty().inner_named_type()));
        if let Some(nested) = &field.selection_set {
            resolve_fields(schema, nested);
        }
    }
}

/// A schema with 2,000 object types reachable from the root, and an operation
/// that selects all of them. Validation and the field walk are dominated by
/// name lookups.
fn bench_wide_operation(c: &mut Criterion) {
    const TYPES: usize = 2_000;
    let mut sdl = String::from("type Query {\n");
    for i in 0..TYPES {
        _ = writeln!(&mut sdl, "  t{i}: T{i}");
    }
    sdl.push_str("}\n");
    for i in 0..TYPES {
        let next = (i + 1) % TYPES;
        _ = writeln!(
            &mut sdl,
            "type T{i} {{ id: ID! name{i}: String next: T{next} }}"
        );
    }
    let mut query = String::from("{\n");
    for i in 0..TYPES {
        let next = (i + 1) % TYPES;
        _ = writeln!(
            &mut query,
            "  t{i} {{ id name{i} next {{ id name{next} }} }}"
        );
    }
    query.push_str("}\n");
    let schema = Schema::parse_and_validate(sdl, "schema.graphql").unwrap();

    c.bench_function("wide operation parse_and_validate", |b| {
        b.iter(|| {
            let doc =
                ExecutableDocument::parse_and_validate(&schema, &query, "query.graphql").unwrap();
            std::hint::black_box(doc);
        })
    });

    let doc = ExecutableDocument::parse_and_validate(&schema, &query, "query.graphql").unwrap();
    c.bench_function("wide operation resolve fields", |b| {
        b.iter(|| {
            for operation in doc.operations.iter() {
                resolve_fields(&schema, &operation.selection_set);
            }
        })
    });
}

criterion_group!(
    lookups,
    bench_supergraph,
    bench_many_types,
    bench_wide_operation
);
criterion_main!(lookups);
