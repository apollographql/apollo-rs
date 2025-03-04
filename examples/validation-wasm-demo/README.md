# WebAssembly demo of GraphQL validation

1. [Install wasm-pack](https://rustwasm.github.io/wasm-pack/installer/)
2. Install either [miniserve](https://crates.io/crates/miniserve) with `cargo install miniserve`,
   or Python
3. Move to this directory if needed: `cd examples/validation-wasm-demo`
3. Build with `wasm-pack build --target web`
4. Start an HTTP server with `miniserve --index index.html` or `python3 -m http.server`
5. Navigate to [http://127.0.0.1:8080/](http://127.0.0.1:8080/)