use std::io::Read;
use std::process::ExitCode;

/// A simple program to run the validations implemented by apollo-compiler
/// and report any errors.
///
/// To use, do:
/// cargo run --example validate path/to/input.graphql
fn main() -> ExitCode {
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

    match apollo_compiler::parse_mixed_validate(source, filename) {
        Ok((_schema, _executable)) => ExitCode::SUCCESS,
        Err(errors) => {
            eprintln!("{errors:?}");
            ExitCode::FAILURE
        }
    }
}
