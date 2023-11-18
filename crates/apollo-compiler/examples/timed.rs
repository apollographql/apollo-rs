//! Parse and validate a schema and executable document provided as files.
//! Print the time taken by each step.

use apollo_compiler::ExecutableDocument;
use apollo_compiler::Schema;
use std::process::ExitCode;
use std::time::Instant;

fn main() -> ExitCode {
    let mut args = std::env::args_os();
    let _arg_0 = args.next(); // filename of this program
    let arg_1 = args.next();
    let arg_2 = args.next();
    let (Some(schema_filename), Some(executable_filename)) = (arg_1, arg_2) else {
        eprintln!(
            "Usage: cargo run --release --example timed <schema.graphql> <executable.graphql>"
        );
        return ExitCode::FAILURE;
    };

    let schema_source = std::fs::read_to_string(&schema_filename).unwrap();
    let executable_source = std::fs::read_to_string(&executable_filename).unwrap();

    let step = format!("Schema parse ({} bytes)", schema_source.len());
    let schema = match timed(&step, || Schema::parse(schema_source, schema_filename)) {
        Ok(s) => s,
        Err(errors) => {
            println!("Schema parse errors:\n{errors}");
            return ExitCode::FAILURE;
        }
    };

    let schema = match timed("Schema validation", || schema.validate()) {
        Ok(s) => s,
        Err(errors) => {
            println!("Schema is invalid:\n{errors}");
            return ExitCode::FAILURE;
        }
    };

    let doc = match timed(
        &format!(
            "Executable document parse ({} bytes)",
            executable_source.len()
        ),
        || ExecutableDocument::parse(&schema, executable_source, executable_filename),
    ) {
        Ok(doc) => doc,
        Err(errors) => {
            println!("Executable document parse errors:\n{errors}");
            return ExitCode::FAILURE;
        }
    };

    if let Err(errors) = timed("Executable document validation", || doc.validate(&schema)) {
        println!("Executable document is invalid:\n{errors}")
    }

    ExitCode::SUCCESS
}

fn timed<T>(step: &str, f: impl FnOnce() -> T) -> T {
    let start = Instant::now();
    let result = f();
    let elapsed = start.elapsed();
    println!("{step}: {:.3} ms", elapsed.as_secs_f32() * 1_000.);
    result
}
