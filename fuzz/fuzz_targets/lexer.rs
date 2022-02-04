#![no_main]
use apollo_parser::Parser;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &str| {
    let _parser = Parser::new(data);
});
