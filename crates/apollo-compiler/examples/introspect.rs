use apollo_compiler::execution::SchemaIntrospectionQuery;
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

    let query = if filename.starts_with('{') {
        ExecutableDocument::parse_and_validate(&schema, filename, "input.graphql")
    } else {
        ExecutableDocument::parse_and_validate(
            &schema,
            std::fs::read_to_string(&filename)?,
            filename,
        )
    }
    .map_err(|err| err.errors.to_string())?;

    let variables = Default::default();
    let response = SchemaIntrospectionQuery::split_and_execute(
        &schema,
        &query,
        query
            .operations
            .get(None)
            .map_err(|_| "Provided query must be an anonymous introspection query")?,
        Valid::assume_valid_ref(&variables),
        |_| panic!("Provided query must not have non-introspection elements"),
    );

    serde_json::to_writer_pretty(std::io::stdout().lock(), &response)?;

    Ok(())
}
