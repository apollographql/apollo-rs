//! Output fields may be typed as any output type: objects, interfaces,
//! unions, enums, and scalars. These tests check that the generator
//! produces abstract and custom scalar field types, and that it can
//! select into them without producing invalid operations.

use apollo_compiler::ast::Definition;
use apollo_compiler::ast::Document as AstDocument;
use apollo_compiler::ast::Selection;
use apollo_smith::DocumentBuilder;
use arbitrary::Unstructured;

fn bytes(seed: u64, len: usize) -> Vec<u8> {
    let mut x = seed.wrapping_mul(0x9E3779B97F4A7C15) | 1;
    (0..len)
        .map(|_| {
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            x as u8
        })
        .collect()
}

// Objects that declare the same interface more than once are a separate,
// known generator bug, so they are filtered out here.
fn unexpected_errors(doc: &str) -> Vec<String> {
    let ast = AstDocument::parse(doc, "smith.graphql").expect("smith output must parse");
    match ast.to_mixed_validate() {
        Ok(_) => vec![],
        Err(errors) => errors
            .iter()
            .map(|e| e.error.to_string())
            .filter(|m| !(m.contains("implements interface") && m.contains("more than once")))
            .collect(),
    }
}

fn operations_for_schema(schema: &str) {
    for seed in 1..=200u64 {
        let data = bytes(seed, 512);
        let mut u = Unstructured::new(&data);
        let doc = apollo_parser::Parser::new(schema)
            .parse()
            .document()
            .try_into()
            .expect("schema converts to a smith document");
        let mut builder = DocumentBuilder::with_document(&mut u, doc).expect("builder from schema");
        let Some(op) = builder
            .operation_definition()
            .expect("operation generation")
        else {
            continue;
        };
        let op: String = op.into();
        let full = format!("{schema}\n{op}");
        let errors = unexpected_errors(&full);
        assert!(errors.is_empty(), "seed={seed}\n{full}\n{errors:#?}");
    }
}

#[test]
fn selects_into_union_typed_fields() {
    operations_for_schema(
        "schema { query: Query }
        type Query { u: U }
        union U = A | B
        type A { a: Int, shared: Int }
        type B { b: String, shared: String }",
    );
}

#[test]
fn selects_into_custom_scalar_typed_fields() {
    operations_for_schema(
        "schema { query: Query }
        type Query { s: Custom, i: Int }
        scalar Custom",
    );
}

#[test]
fn generates_interface_and_custom_scalar_field_types() {
    let mut interface_fields = 0;
    let mut custom_scalar_fields = 0;
    for seed in 1..=50u64 {
        let data = bytes(seed, 16384);
        let mut u = Unstructured::new(&data);
        let doc: String = DocumentBuilder::new(&mut u).build().expect("build").into();
        let ast = AstDocument::parse(&doc, "smith.graphql").expect("smith output must parse");
        let mut interfaces = Vec::new();
        let mut scalars = Vec::new();
        for def in &ast.definitions {
            match def {
                Definition::InterfaceTypeDefinition(d) => interfaces.push(d.name.clone()),
                Definition::ScalarTypeDefinition(d) => scalars.push(d.name.clone()),
                _ => {}
            }
        }
        for def in &ast.definitions {
            let fields = match def {
                Definition::ObjectTypeDefinition(d) => &d.fields,
                Definition::InterfaceTypeDefinition(d) => &d.fields,
                _ => continue,
            };
            for field in fields {
                let inner = field.ty.inner_named_type();
                interface_fields += interfaces.contains(inner) as usize;
                custom_scalar_fields += scalars.contains(inner) as usize;
            }
        }
        let errors = unexpected_errors(&doc);
        assert!(errors.is_empty(), "seed={seed}\n{doc}\n{errors:#?}");
    }
    assert!(interface_fields > 0, "no field was typed as an interface");
    assert!(
        custom_scalar_fields > 0,
        "no field was typed as a custom scalar"
    );
}

fn operations(schema: &str, seeds: u64) -> Vec<(u64, AstDocument)> {
    (1..=seeds)
        .filter_map(|seed| {
            let data = bytes(seed, 512);
            let mut u = Unstructured::new(&data);
            let doc = apollo_parser::Parser::new(schema)
                .parse()
                .document()
                .try_into()
                .expect("schema converts to a smith document");
            let mut builder =
                DocumentBuilder::with_document(&mut u, doc).expect("builder from schema");
            let op: String = builder
                .operation_definition()
                .expect("operation generation")?
                .into();
            let full = format!("{schema}\n{op}");
            let errors = unexpected_errors(&full);
            assert!(errors.is_empty(), "seed={seed}\n{full}\n{errors:#?}");
            Some((
                seed,
                AstDocument::parse(&full, "smith.graphql").expect("parse"),
            ))
        })
        .collect()
}

// Counts `...FB` spread directly in a selection on `U`, not after
// narrowing to `B` with an inline fragment.
fn spreads_on_u(selections: &[Selection]) -> usize {
    fn direct(selections: &[Selection]) -> usize {
        selections
            .iter()
            .map(|s| match s {
                Selection::FragmentSpread(f) => (f.fragment_name == "FB") as usize,
                Selection::InlineFragment(i) if i.type_condition.is_none() => {
                    direct(&i.selection_set)
                }
                _ => 0,
            })
            .sum()
    }
    selections
        .iter()
        .map(|s| match s {
            Selection::Field(f) if f.name == "u" => f.selection_set.as_deref().map_or(0, direct),
            Selection::Field(f) => f.selection_set.as_deref().map_or(0, spreads_on_u),
            Selection::InlineFragment(i) => spreads_on_u(&i.selection_set),
            Selection::FragmentSpread(_) => 0,
        })
        .sum()
}

#[test]
fn spreads_fragments_on_members_added_by_union_extensions() {
    let schema = "schema { query: Query }
        type Query { u: U }
        union U = A
        extend union U = B
        type A { a: Int }
        type B { b: Int }
        fragment FB on B { b }";
    let mut spreads = 0;
    for seed in 1..=200u64 {
        let data = bytes(seed, 512);
        let mut u = Unstructured::new(&data);
        let doc = apollo_parser::Parser::new(schema)
            .parse()
            .document()
            .try_into()
            .expect("schema converts to a smith document");
        let mut builder = DocumentBuilder::with_document(&mut u, doc).expect("builder from schema");
        let Some(op) = builder
            .operation_definition()
            .expect("operation generation")
        else {
            continue;
        };
        let op: String = op.into();
        // FB is only used when this operation happens to spread it
        let full = format!("{schema}\n{op}");
        let ast = AstDocument::parse(&full, "smith.graphql").expect("parse");
        for def in &ast.definitions {
            if let Definition::OperationDefinition(op) = def {
                spreads += spreads_on_u(&op.selection_set);
            }
        }
        let errors: Vec<String> = unexpected_errors(&full)
            .into_iter()
            .filter(|m| !m.contains("`FB`"))
            .collect();
        assert!(errors.is_empty(), "seed={seed}\n{full}\n{errors:#?}");
    }
    assert!(spreads > 0, "FB was never spread into a U selection");
}

// Member types reuse field names with different types. A field that can't
// merge should give way to one that can, rather than to `__typename`.
#[test]
fn conflicting_member_fields_give_way_to_mergeable_ones() {
    let schema = "schema { query: Query }
        type Query { u: U }
        union U = A | B
        type A { x: Int, y: Int }
        type B { x: String, y: Int }";
    for (seed, doc) in operations(schema, 200) {
        let text = doc.serialize().to_string();
        for member in ["A", "B"] {
            let marker = format!("... on {member} {{");
            for (i, _) in text.match_indices(&marker) {
                let body = &text[i + marker.len()..];
                let first_close = body.find('}').expect("inline fragment closes");
                assert!(
                    !body[..first_close].contains("__typename"),
                    "seed={seed}\n{text}"
                );
            }
        }
    }
}
