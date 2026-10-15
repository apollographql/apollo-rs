//! Helpers shared by the integration tests.

// Each test binary uses a different subset of these helpers.
#![allow(dead_code)]

use apollo_smith::DocumentBuilder;
use arbitrary::Unstructured;

/// Deterministic pseudo-random input, so a failing seed can be rerun.
pub fn bytes(seed: u64, len: usize) -> Vec<u8> {
    let mut x = seed.wrapping_mul(0x9E3779B97F4A7C15) | 1;
    (0..len)
        .map(|_| {
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            x as u8
        })
        .collect()
}

/// Generate an operation against `schema` from 512 bytes of input for
/// `seed`, or `None` when the builder doesn't produce one.
pub fn operation_for_schema(schema: &str, seed: u64) -> Option<String> {
    let data = bytes(seed, 512);
    let mut u = Unstructured::new(&data);
    let doc = apollo_parser::Parser::new(schema)
        .parse()
        .document()
        .try_into()
        .expect("schema converts to a smith document");
    let mut builder = DocumentBuilder::with_document(&mut u, doc).expect("builder from schema");
    builder
        .operation_definition()
        .expect("operation generation")
        .map(String::from)
}
