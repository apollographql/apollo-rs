use apollo_compiler::ast;
use apollo_smith::DocumentBuilder;
use arbitrary::Unstructured;

const SCHEMA: &str = "schema { query: Query } type Query { id: ID }";

#[test]
fn generates_valid_empty_selection_set() {
    let schema = apollo_parser::Parser::new(SCHEMA)
        .parse()
        .document()
        .try_into()
        .expect("schema converts to a smith document");
    // Exhausted input picks the minimum selection count, which is zero.
    let mut u = Unstructured::new(&[]);
    let operation: String = DocumentBuilder::with_document(&mut u, schema)
        .expect("builder from schema")
        .operation_definition()
        .expect("operation generation")
        .expect("schema has a query root")
        .into();

    let doc = ast::Document::parse(format!("{SCHEMA}\n{operation}"), "smith.graphql")
        .expect("smith output parses");
    let op = doc
        .definitions
        .iter()
        .find_map(|definition| definition.as_operation_definition())
        .expect("document contains the generated operation");
    assert!(op.selection_set.is_empty(), "generated: {operation}");
    doc.to_mixed_validate()
        .expect("empty selection set validates");
}
