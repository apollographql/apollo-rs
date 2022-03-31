#![no_main]
use apollo_parser::Parser;
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

    let parser = panic::catch_unwind(|| Parser::new(&doc_generated));

    let parser = match parser {
        Err(err) => {
            panic!("error {:?}", err);
        }
        Ok(p) => p,
    };
    debug!("======= DOCUMENT =======");
    debug!("{}", doc_generated);
    debug!("========================");

    let tree = parser.parse();
    // early return if the parser detected an error
    let mut should_panic = false;
    if tree.errors().len() > 0 {
        should_panic = true;
        let errors = tree
            .errors()
            .map(|err| err.message())
            .collect::<Vec<&str>>()
            .join("\n");
        debug!("Parser errors ========== \n{:?}", errors);
        debug!("========================");
        log_gql_doc(&doc_generated, &errors);
    }
    if should_panic {
        panic!("error detected");
    }
});
