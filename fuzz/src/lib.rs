use apollo_smith::DocumentBuilder;
use libfuzzer_sys::arbitrary::Result;
use libfuzzer_sys::arbitrary::Unstructured;

/// This generate an arbitrary valid GraphQL document
pub fn generate_valid_document(input: &[u8]) -> Result<String> {
    drop(env_logger::try_init());

    let mut u = Unstructured::new(input);
    let document = DocumentBuilder::new(&mut u).build()?;

    Ok(document.into())
}

/// Log the error and the document generated for these errors
/// Save it into files
pub fn log_gql_doc(gql_doc: &str, errors: &str) {
    log::debug!("writing test case to test.graphql ...");
    std::fs::write("test_case.graphql", gql_doc).unwrap();
    std::fs::write("test_case_error.log", errors).unwrap();
}

/// Call the CST accessors that read token text, so fuzz targets also cover
/// trees built from invalid input.
pub fn read_cst(node: &apollo_parser::SyntaxNode) {
    use apollo_parser::cst;
    use apollo_parser::cst::CstNode;

    for node in node.descendants() {
        if let Some(name) = cst::Name::cast(node.clone()) {
            let _ = name.text();
        } else if let Some(variable) = cst::Variable::cast(node.clone()) {
            let _ = variable.text();
        } else if let Some(value) = cst::EnumValue::cast(node.clone()) {
            let _ = value.text();
        } else if let Some(value) = cst::StringValue::cast(node.clone()) {
            let _ = String::from(value);
        } else if let Some(value) = cst::IntValue::cast(node.clone()) {
            let _ = i32::try_from(&value);
            let _ = f64::try_from(value);
        } else if let Some(value) = cst::FloatValue::cast(node.clone()) {
            let _ = f64::try_from(value);
        } else if let Some(value) = cst::BooleanValue::cast(node.clone()) {
            let _ = bool::try_from(value);
        }
    }
}
