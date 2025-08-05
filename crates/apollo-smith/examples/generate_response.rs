use apollo_compiler::validation::Valid;
use apollo_compiler::ExecutableDocument;
use apollo_compiler::Schema;
use apollo_smith::ResponseBuilder;
use arbitrary::Result;
use arbitrary::Unstructured;
use rand::Rng;
use serde_json_bytes::Value;
use std::fs;

pub fn generate_valid_response(
    doc: &Valid<ExecutableDocument>,
    schema: &Valid<Schema>,
) -> Result<Value> {
    let mut buf = [0u8; 2048];
    rand::rng().fill(&mut buf);
    let mut u = Unstructured::new(&buf);

    ResponseBuilder::new(&mut u, doc, schema)
        .with_min_list_size(2)
        .with_max_list_size(10)
        .build()
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let Some(schema_path) = args.next() else {
        return Err("Provide a schema path".into());
    };
    let schema = fs::read_to_string(schema_path.clone())
        .map_err(|e| format!("Failed to read schema file: {}", e))?;
    let schema = Schema::parse_and_validate(&schema, &schema_path)
        .map_err(|e| format!("Failed to parse schema: {}", e))?;

    let Some(doc_path) = args.next() else {
        return Err("Provide a document path".into());
    };
    let doc = fs::read_to_string(doc_path.clone())
        .map_err(|e| format!("Failed to read document file: {}", e))?;
    let doc = ExecutableDocument::parse_and_validate(&schema, &doc, &doc_path)
        .map_err(|e| format!("Failed to parse document: {}", e))?;

    let response = generate_valid_response(&doc, &schema)?;
    println!("Generated response: {response}");
    Ok(())
}
