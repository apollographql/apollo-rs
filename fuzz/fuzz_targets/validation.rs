#![no_main]
use apollo_compiler::{ExecutableDocument, Schema};
use apollo_rs_fuzz::generate_valid_operation;
use libfuzzer_sys::fuzz_target;
use log::debug;
use router_bridge;
// use serde_json::json;
// use serde_json::Value;
use std::panic;

fuzz_target!(|data: &[u8]| {
    let (op, ts) = match generate_valid_operation(data) {
        Ok(d) => (d.0, d.1),
        Err(_err) => {
            return;
        }
    };

    let schema = Schema::parse(&ts, "ts.graphql");
    let executable = ExecutableDocument::parse(&schema, &op, "op.graphql");
    let rust_diagnostics = executable.validate(&schema);

    debug!("======= OPERATION =======");
    debug!("{}", op);
    debug!("========= SCHEMA ===============");
    debug!("{}", ts);
    debug!("========================");
    let js_diagnostics = router_bridge::validate::validate(&ts, &op)
        .expect("bridge should return validation result");

    // early return if js and rust validation errors don't match
    let mut should_panic = false;
    match js_diagnostics.errors.clone() {
        Some(js_diag) => match rust_diagnostics {
            Ok(_) => {
                should_panic = true;
                debug!("JS ERRORS FOUND BUT NOT RUST");
                for diag in js_diag {
                    debug!("{}", diag);
                }
            }
            Err(rust_diagnostics) => {
                if rust_diagnostics.len() != js_diag.len() {
                    should_panic = true;
                    debug!("======== UNMATCHED DIAGNOSTICS LEN BETWEEN RUST & JS ======= ");
                    for diag in js_diag {
                        debug!("JS DIAG: {}", diag)
                    }
                    for diag in rust_diagnostics.iter() {
                        debug!("RUST DIAG: {}", diag)
                    }
                }
            }
        },
        None => {
            if rust_diagnostics.is_err() {
                should_panic = true;
                debug!("======== RUST ERRORS FOUND BUT NOT JS ======= ");
                rust_diagnostics.map_err(|diags| diags.iter().map(|diag| debug!("{}", diag)));
            }
        }
    }

    debug!("========== RUST DIAGNOSTICS ==============");
    debug!("{:?}", rust_diagnostics);
    // for diag in rust_diagnostics {
    //     debug!("{}", diag);
    // }

    debug!("========== JS DIAGNOSTICS ==============");
    debug!("{:?}", js_diagnostics.errors);

    if should_panic {
        panic!("error detected");
    }
});
