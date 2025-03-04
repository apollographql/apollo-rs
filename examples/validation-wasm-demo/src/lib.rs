use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn validate(schema_sdl: &str, executable_document: &str) -> Option<String> {
    let schema_result = apollo_compiler::Schema::parse_and_validate(schema_sdl, "schema.graphql");
    if executable_document.trim().is_empty() {
        return schema_result.err().map(|e| e.to_string());
    }
    let schema = match &schema_result {
        Ok(s) => s,
        Err(with_errors) => {
            apollo_compiler::validation::Valid::assume_valid_ref(&with_errors.partial)
        }
    };
    let executable_result = apollo_compiler::ExecutableDocument::parse_and_validate(
        schema,
        executable_document,
        "executable.graphql",
    );
    match (schema_result, executable_result) {
        (Ok(_), Ok(_)) => None,
        (Ok(_), Err(e)) => Some(e.to_string()),
        (Err(e), Ok(_)) => Some(e.to_string()),
        (Err(mut e1), Err(e2)) => {
            e1.errors.merge(e2.errors);
            Some(e1.to_string())
        }
    }
}
