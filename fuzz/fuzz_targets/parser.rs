#![no_main]
use apollo_parser::Parser;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let s = match std::str::from_utf8(data) {
        Err(_) => return,
        Ok(s) => s,
    };

    let parser = Parser::new(s);
    let tree = parser.parse();

    // early return if the lexer detected an error
    if tree.errors().next().is_some() {
        return;
    }

    let document = tree.document();
});
