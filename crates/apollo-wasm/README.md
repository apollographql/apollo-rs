# `apollo-wasm`

This crate provides a WASM build for `apollo-compiler` and generates TypeScript/JavaScript bindings for it.

## Prerequisites

1. Install [`rustup`](https://rustup.rs)
1. Install [`wasm-pack`](https://rustwasm.github.io/wasm-pack/installer/)

## Build

`wasm-pack build` will build the WASM package and the TS/JS bindings and output them to `./pkg`.

## Usage

Currently the API exposes a single `validate_document` function that takes in a document and the path to a document. The path can be the empty string if it is not a document from the file system or if it does not have a name.
