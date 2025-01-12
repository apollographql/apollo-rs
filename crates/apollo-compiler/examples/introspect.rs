use apollo_compiler::introspection;
use apollo_compiler::validation::Valid;
use apollo_compiler::ExecutableDocument;
use apollo_compiler::Schema;

/// To use, do:
/// cargo run --example introspect path/to/schema.graphql introspection_query.graphql
///
/// You can also provide simple queries inline:
/// cargo run --example introspect path/to/schema.graphql '{ __schema { types { name } } }'
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let Some(filename) = args.next() else {
        return Err("Provide a schema".into());
    };
    let schema = Schema::parse_and_validate(std::fs::read_to_string(&filename)?, filename)
        .map_err(|err| err.errors.to_string())?;

    let Some(filename) = args.next() else {
        return Err("Provide a query to execute".into());
    };

    let doc = if filename.starts_with('{') {
        ExecutableDocument::parse_and_validate(&schema, filename, "input.graphql")
    } else {
        ExecutableDocument::parse_and_validate(
            &schema,
            std::fs::read_to_string(&filename)?,
            filename,
        )
    }
    .map_err(|err| err.errors.to_string())?;

    let response = introspection::partial_execute(
        &schema,
        &schema.implementers_map(),
        &doc,
        doc.operations
            .get(None)
            .map_err(|_| "Must have exactly one operation")?,
        Valid::assume_valid_ref(&Default::default()),
    )
    .map_err(|e| e.message().to_string())?;

    serde_json::to_writer_pretty(std::io::stdout().lock(), &response)?;

    Ok(())
}
