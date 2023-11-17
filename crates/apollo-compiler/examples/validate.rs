use std::io::Read;

/// A simple program to run the validations implemented by apollo-compiler
/// and report any errors.
///
/// To use, do:
/// cargo run --example validate path/to/input.graphql
fn main() {
    let (source, filename) = match std::env::args().nth(1).as_deref() {
        Some("-") | None => {
            let mut source = String::new();
            std::io::stdin().read_to_string(&mut source).unwrap();
            (source, "stdin.graphql".to_string())
        }
        Some(filename) => (
            std::fs::read_to_string(filename).unwrap(),
            filename.to_string(),
        ),
    };

    let (schema, executable) = apollo_compiler::parse_mixed(source, filename);
    let schema_result = schema.validate();
    let executable_result = executable.validate(&schema);
    let has_errors = schema_result.is_err() || executable_result.is_err();
    match schema_result {
        Ok(()) => {}
        Err(errors) => println!("{errors}"),
    }
    match executable_result {
        Ok(()) => {}
        Err(errors) => println!("{errors}"),
    }

    std::process::exit(if has_errors { 1 } else { 0 });
}
