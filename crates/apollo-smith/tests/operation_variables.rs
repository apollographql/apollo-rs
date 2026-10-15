//! Operations define variables and pass them as field arguments. Every
//! variable must be defined, used, and passed only where its type is
//! allowed.

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

#[test]
fn operations_use_valid_variables() {
    let mut operations_with_variables = 0;
    for seed in 1..=50u64 {
        let data = bytes(seed, 16384);
        let mut u = Unstructured::new(&data);
        // Few, small types leave input for operations, which are generated
        // after every type definition. Some inputs can't produce a document
        // at all, which is fine here.
        let Ok(doc) = DocumentBuilder::new(&mut u)
            .max_scalar_types(2)
            .max_enum_types(2)
            .max_interface_types(2)
            .max_object_types(3)
            .max_union_types(2)
            .max_input_object_types(2)
            .max_fragment_definitions(2)
            .max_directive_definitions(2)
            .build()
        else {
            continue;
        };
        let doc: String = doc.into();
        let ast = AstDocument::parse(&doc, "smith.graphql").expect("smith output must parse");
        operations_with_variables += ast
            .definitions
            .iter()
            .filter(
                |d| matches!(d, Definition::OperationDefinition(op) if !op.variables.is_empty()),
            )
            .count();
        if let Err(errors) = ast.to_mixed_validate() {
            let variable_errors: Vec<String> = errors
                .iter()
                .map(|e| e.error.to_string())
                .filter(|m| m.contains("variable") || m.contains('$'))
                .collect();
            assert!(
                variable_errors.is_empty(),
                "seed={seed}\n{doc}\n{variable_errors:#?}"
            );
        }
    }
    assert!(
        operations_with_variables > 0,
        "no operation defined a variable"
    );
}

#[test]
fn with_document_operations_use_valid_variables() {
    let schema = "schema { query: Query }
        type Query { f(a: Int, b: String!, c: [Int!]): Int, g(a: Int!): Query }";
    let mut operations_with_variables = 0;
    for seed in 1..=100u64 {
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
        operations_with_variables += op.contains('$') as usize;
        let full = format!("{schema}\n{op}");
        let ast = AstDocument::parse(&full, "smith.graphql").expect("smith output must parse");
        if let Err(errors) = ast.to_mixed_validate() {
            panic!("seed={seed}\n{full}\n{errors}");
        }
    }
    assert!(
        operations_with_variables > 0,
        "no operation defined a variable"
    );
}
