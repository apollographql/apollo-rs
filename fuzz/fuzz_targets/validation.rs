#![no_main]
use apollo_compiler::ApolloCompiler;
use apollo_rs_fuzz::{generate_valid_document, log_gql_doc};
use libfuzzer_sys::fuzz_target;
use log::debug;
use std::panic;

fuzz_target!(|data: &[u8]| {
    let doc_generated = match generate_valid_document(data) {
        Ok(d) => d,
        Err(_err) => {
            return;
        }
    };

    let mut compiler = ApolloCompiler::new();
    compiler.add_document(&doc_generated, "fuzz.graphql");

    debug!("======= DOCUMENT =======");
    debug!("{}", doc_generated);
    debug!("========================");

    let diagnostics = panic::catch_unwind(|| compiler.validate());
    match diagnostics {
        Ok(diagnostics) => {
            debug!("===== Diagnostics ======");
            for diag in diagnostics {
                debug!("{}", diag);
            }
            debug!("========================");
        },
        Err(panic) => {
            let err_str = panic.downcast_ref::<String>()
                .map(|err| err.to_string())
                .unwrap_or_default();
            log_gql_doc(&doc_generated, &err_str);
            panic::panic_any(panic);
        }
    }
});
