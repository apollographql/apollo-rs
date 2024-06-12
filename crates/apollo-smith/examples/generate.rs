use std::fs;

use apollo_parser::Parser;
use apollo_smith::Document;
use apollo_smith::DocumentBuilder;
use arbitrary::Result;
use arbitrary::Unstructured;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};

/// This generate an arbitrary valid GraphQL operation
pub fn generate_valid_operation(schema_path: &str, seed_arg: Option<String>) -> Result<String> {
    let contents = fs::read_to_string(schema_path).expect("cannot read file");
    let parser = Parser::new(&contents);

    let tree = parser.parse();
    if tree.errors().len() > 0 {
        let errors = tree
            .errors()
            .map(|err| err.message())
            .collect::<Vec<&str>>()
            .join("\n");
        panic!("cannot parse the supergraph:\n{errors}");
    }

    let seed: String = match seed_arg {
        Some(s) => s,
        None => thread_rng()
            .sample_iter(&Alphanumeric)
            .take(30)
            .map(char::from)
            .collect(),
    };

    println!("generating from seed: {seed}");
    let mut u = Unstructured::new(seed.as_bytes());
    let mut gql_doc = DocumentBuilder::with_document(
        &mut u,
        Document::try_from(tree.document()).expect("tree should not have errors"),
    )?;
    let operation_def: String = gql_doc.operation_definition()?.unwrap().into();

    Ok(operation_def)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let Some(schema_path) = args.next() else {
        return Err("Provide a schema path".into());
    };
    let seed = args.next();

    let operation_def = generate_valid_operation(&schema_path, seed)?;
    println!("operation definition:\n{operation_def}");
    Ok(())
}
