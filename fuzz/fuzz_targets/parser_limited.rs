#![no_main]
use apollo_parser::Parser;
use libfuzzer_sys::fuzz_target;
use std::panic;

// Use completely arbitrary input and a token limit to find cases where the limit
// being reached causes a loop in the parser.
fuzz_target!(|data: &str| {
    let _ = env_logger::try_init();

    let parser = panic::catch_unwind(|| Parser::new(data));
    let parser = match parser {
        Err(err) => {
            panic!("error {err:?}");
        }
        Ok(p) => p.token_limit(500),
    };

    // We expect to have errors--we just want to make sure it does not crash.
    let _tree = parser.parse();
});
