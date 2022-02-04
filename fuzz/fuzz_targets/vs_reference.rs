#![no_main]
use apollo_parser::Parser;
use libfuzzer_sys::fuzz_target;
use std::panic;
use std::process::Command;

fuzz_target!(|data: &str| {
    if data.chars().any(|c| c == '\0') {
        // We can't pass in strings with null chars.
        return;
    }

    let parser = panic::catch_unwind(|| Parser::new(data));

    let parser = match parser {
        Err(_) => return,
        Ok(p) => p,
    };

    let tree = parser.parse();

    // Now let's input it against the reference js implementation:
    let output = Command::new("node")
        .arg("fuzz/js_reference_impl/index.js")
        .arg(data)
        .output()
        .expect("failed to execute node");

    if std::str::from_utf8(&output.stderr)
        .unwrap()
        .contains("Cannot find module")
    {
        dbg!(std::str::from_utf8(&output.stderr).unwrap());
        panic!("Please run NPM install in fuzz/js_reference_impl");
    }

    let rs_errors: Vec<_> = tree.errors().collect();
    let rs_success = rs_errors.len() == 0;

    if rs_success != output.status.success() {
        dbg!(std::str::from_utf8(&output.stdout).unwrap());
        dbg!(std::str::from_utf8(&output.stderr).unwrap());
        dbg!(rs_errors);
        panic!("Sucess of rust implementation doesn't match the success of the reference js implementation.");
    }
});
