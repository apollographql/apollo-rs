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

    let lexer = panic::catch_unwind(|| Lexer::new(&doc_generated));

    let lexer = match lexer {
        Err(err) => {
            panic!("error {:?}", err);
        }
        Ok(p) => p,
    };

    // early return if the lexer detected an error
    let mut should_panic = false;
    if lexer.errors().len() > 0 {
        should_panic = true;
        let errors = lexer
            .errors()
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
