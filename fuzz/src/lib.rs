use apollo_smith::DocumentBuilder;

use libfuzzer_sys::arbitrary::{Result, Unstructured};

/// This generates an arbitrary valid GraphQL document
pub fn generate_valid_document(input: &[u8]) -> Result<String> {
    drop(env_logger::try_init());

    let mut u = Unstructured::new(input);
    let gql_doc = DocumentBuilder::new(&mut u)?;
    let document = gql_doc.finish();

    Ok(document.into())
}

pub fn generate_schema_document(
    input: &[u8],
) -> std::result::Result<(), apollo_smith::next::Error> {
    drop(env_logger::try_init());

    apollo_smith::next::generate_schema_document(&mut apollo_smith::next::Unstructured::new(
        input,
    ))?;
    Ok(())
}

/// Log the error and the document generated for these errors
/// Save it into files
pub fn log_gql_doc(gql_doc: &str, errors: &str) {
    log::debug!("writing test case to test.graphql ...");
    std::fs::write("test_case.graphql", gql_doc).unwrap();
    std::fs::write("test_case_error.log", errors).unwrap();
}
