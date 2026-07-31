#![no_main]
use apollo_compiler::ast::Document;
use apollo_rs_fuzz::generate_valid_document;
use apollo_rs_fuzz::log_gql_doc;
use libfuzzer_sys::fuzz_target;
use log::debug;

fuzz_target!(|data: &[u8]| {
    let _ = env_logger::try_init();

    let doc_generated = match generate_valid_document(data) {
        Ok(d) => d,
        Err(_) => return,
    };

    debug!("======= DOCUMENT =======");
    debug!("{doc_generated}");
    debug!("========================");

    let ast = match Document::parse(&doc_generated, "smith.graphql") {
        Ok(ast) => ast,
        Err(with_errors) => {
            let errors = with_errors.errors.to_string();
            log_gql_doc(&doc_generated, &errors);
            panic!("apollo-smith produced un-parseable document:\n{errors}");
        }
    };

    if let Err(errors) = ast.to_mixed_validate() {
        let errors = errors.to_string();
        log_gql_doc(&doc_generated, &errors);
        panic!("apollo-smith produced invalid GraphQL:\n{errors}");
    }
});
