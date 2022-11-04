#![no_main]
use apollo_parser::Lexer;
use apollo_rs_fuzz::{generate_valid_document, log_gql_doc};
use libfuzzer_sys::fuzz_target;
use log::debug;
use std::panic;

fuzz_target!(|data: &[u8]| {
    let doc_generated = match generate_valid_document(data) {
        Ok(d) => d,
        Err(_) => {
            return;
        }
    };

    let lexer = panic::catch_unwind(|| Lexer::new(&doc_generated).lex());

    let (_tokens, errors) = match lexer {
        Err(err) => {
            panic!("error {:?}", err);
        }
        Ok(p) => p,
    };
    debug!("======= DOCUMENT =======");
    debug!("{}", doc_generated);
    debug!("========================");

    // early return if the lexer detected an error
    let mut should_panic = false;
    if !errors.is_empty() {
        should_panic = true;
        let errors = errors
            .iter()
            .map(|err| err.message())
            .collect::<Vec<&str>>()
            .join("\n");
        debug!("======= DOCUMENT =======");
        debug!("{}", doc_generated);
        debug!("========================");
        debug!("Lexer errors =========== \n{:?}", errors);
        debug!("========================");
        log_gql_doc(&doc_generated, &errors);
    }
    if should_panic {
        panic!("error detected");
    }
});
