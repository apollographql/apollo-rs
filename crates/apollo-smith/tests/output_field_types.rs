//! Output fields may be typed as any output type: objects, interfaces,
//! unions, enums, and scalars. These tests check that the generator
//! produces abstract and custom scalar field types, and that it can
//! select into them without producing invalid operations.

use apollo_compiler::ast::Definition;
use apollo_compiler::ast::Document as AstDocument;
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

// Objects that implement an interface through more than one extension
// are a separate, known generator bug, so they are filtered out here.
fn unexpected_errors(doc: &str) -> Vec<String> {
    let ast = AstDocument::parse(doc, "smith.graphql").expect("smith output must parse");
    match ast.to_mixed_validate() {
        Ok(_) => vec![],
        Err(errors) => errors
            .iter()
            .map(|e| e.error.to_string())
            .filter(|m| !m.contains("more than once"))
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
