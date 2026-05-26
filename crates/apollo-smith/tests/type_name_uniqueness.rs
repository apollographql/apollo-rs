//! Generated documents must not reuse the same name for two type definitions
//! (across object/interface/union/scalar/enum/input-object), because the
//! GraphQL spec puts all of them in a single namespace.

use apollo_compiler::ast::Document as AstDocument;
use apollo_smith::DocumentBuilder;
use arbitrary::Unstructured;

fn assert_type_names_unique(seed: &[u8]) {
    let mut u = Unstructured::new(seed);
    let doc: String = DocumentBuilder::new(&mut u).unwrap().finish().into();
    let ast = AstDocument::parse(&doc, "smith.graphql").expect("smith output must parse");

    if let Err(errors) = ast.to_mixed_validate() {
        let errors_str = errors.to_string();
        assert!(
            !errors_str.contains("is defined multiple times in the schema"),
            "seed={seed:?}: smith produced colliding type names\n\nErrors:\n{errors_str}\n\nGenerated doc:\n{doc}"
        );
    }
}

#[test]
fn empty_seed() {
    assert_type_names_unique(&[]);
}

#[test]
fn small_sequential_seeds() {
    for n in [1usize, 4, 10, 64, 256] {
        let seed: Vec<u8> = (0..n).map(|i| i as u8).collect();
        assert_type_names_unique(&seed);
    }
}
