#![no_main]
use apollo_parser::Parser;
use libfuzzer_sys::fuzz_target;
use std::panic;

fuzz_target!(|data: &str| {
    let parser = panic::catch_unwind(|| Parser::new(data));

    let parser = match parser {
        Err(_) => return,
        Ok(p) => p,
    };

    let tree = parser.parse();

    // early return if the lexer detected an error
    if tree.errors().next().is_some() {
        return;
    }

    let document = tree.document();
});
