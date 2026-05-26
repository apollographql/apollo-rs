//! Per the GraphQL spec, input positions (argument types, input-object fields,
//! variable definitions) must reference an input type — a scalar, enum, or
//! input object. Object, interface, and union types are not valid here.

use apollo_compiler::ast::Document as AstDocument;
use apollo_smith::DocumentBuilder;
use arbitrary::Unstructured;

fn assert_input_positions_are_input_types(seed: &[u8]) {
    let mut u = Unstructured::new(seed);
    let doc: String = DocumentBuilder::new(&mut u).unwrap().finish().into();
    let ast = AstDocument::parse(&doc, "smith.graphql").expect("smith output must parse");

    if let Err(errors) = ast.to_mixed_validate() {
        let errors_str = errors.to_string();
        assert!(
            !errors_str.contains("must be of an input type"),
            "seed={seed:?}: smith picked a non-input type for an input position\n\nErrors:\n{errors_str}\n\nGenerated doc:\n{doc}"
        );
    }
}

#[test]
fn empty_seed() {
    assert_input_positions_are_input_types(&[]);
}

#[test]
fn small_sequential_seeds() {
    for n in [1usize, 4, 10, 64, 256] {
        let seed: Vec<u8> = (0..n).map(|i| i as u8).collect();
        assert_input_positions_are_input_types(&seed);
    }
}
