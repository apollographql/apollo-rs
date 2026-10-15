//! Output fields may be typed as any output type: objects, interfaces,
//! unions, enums, and scalars. These tests check that the generator
//! produces interface and custom scalar field types, and that it can select
//! into them without producing invalid operations.

use apollo_compiler::ast::Definition;
use apollo_compiler::ast::Document as AstDocument;
use apollo_smith::DocumentBuilder;
use arbitrary::Unstructured;
use common::bytes;
use common::operation_for_schema;

mod common;

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

#[test]
fn selects_into_custom_scalar_typed_fields() {
    let schema = "schema { query: Query }
        type Query { s: Custom, i: Int }
        scalar Custom";
    for seed in 1..=200u64 {
        let Some(op) = operation_for_schema(schema, seed) else {
            continue;
        };
        let full = format!("{schema}\n{op}");
        let errors = unexpected_errors(&full);
        assert!(errors.is_empty(), "seed={seed}\n{full}\n{errors:#?}");
    }
}

#[test]
fn generates_interface_and_custom_scalar_field_types() {
    let mut interface_fields = 0;
    let mut custom_scalar_fields = 0;
    for seed in 1..=50u64 {
        let data = bytes(seed, 16384);
        let mut u = Unstructured::new(&data);
        // Some inputs can't produce a document at all, which is fine here.
        let Ok(doc) = DocumentBuilder::new(&mut u).build() else {
            continue;
        };
        let doc: String = doc.into();
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
