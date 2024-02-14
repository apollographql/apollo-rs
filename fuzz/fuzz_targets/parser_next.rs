#![no_main]
use apollo_parser::Parser;
use apollo_rs_fuzz::{generate_valid_document, log_gql_doc};
use libfuzzer_sys::arbitrary::Unstructured;
use libfuzzer_sys::fuzz_target;
use log::debug;
use std::panic;

fuzz_target!(|data: &[u8]| {
    let _ = env_logger::try_init();

    if let Err(e) = apollo_rs_fuzz::generate_schema_document(data) {
        log::info!("error: {:?}", e);
    }
});
