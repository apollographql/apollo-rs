use wee_alloc;
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

use apollo_compiler::ApolloCompiler;
use serde::Serialize;
use tsify::Tsify;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn validate_document(input: &str, path: &str) -> CompilerValidationResult {
    let mut compiler = ApolloCompiler::new();
    compiler.add_document(input, path);
    CompilerValidationResult {
        diagnostics: compiler
            .validate()
            .iter()
            .map(|diagnostic| diagnostic.to_string())
            .collect(),
    }
}

#[derive(Tsify, Serialize)]
#[tsify(into_wasm_abi)]
pub struct CompilerValidationResult {
    diagnostics: Vec<String>,
}
