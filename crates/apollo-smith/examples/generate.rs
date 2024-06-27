use apollo_parser::Parser;
use apollo_smith::Document;
use apollo_smith::DocumentBuilder;
use arbitrary::Result;
use arbitrary::Unstructured;
use rand::distributions::Alphanumeric;
use rand::rngs::StdRng;
use rand::thread_rng;
use rand::Rng;
use rand::SeedableRng;
use std::fs;

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

    let seed: u64 = match seed_arg {
        Some(s) => s.parse().unwrap(),
        None => thread_rng().gen(),
    };

    println!("generating from seed: {seed}");

    let rng: StdRng = SeedableRng::seed_from_u64(seed);
    let data: String = rng
        .sample_iter(&Alphanumeric)
        .take(65536)
        .map(char::from)
        .collect();

    let mut u = Unstructured::new(data.as_bytes());
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
