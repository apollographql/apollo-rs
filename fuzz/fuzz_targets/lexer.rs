#![no_main]
use apollo_parser::Parser;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let s = match std::str::from_utf8(data) {
        Err(_) => return,
        Ok(s) => s,
    };

    let _parser = Parser::new(s);
});
