<div align="center">
  <h1><code>apollo-parser</code></h1>

  <p>
    <strong>A parser for the GraphQL language.</strong>
  </p>
  <p>
    <a href="https://crates.io/crates/apollo-parser">
        <img src="https://img.shields.io/crates/v/apollo-parser.svg?style=flat-square" alt="Crates.io version" />
    </a>
    <a href="https://crates.io/crates/apollo-parser">
        <img src="https://img.shields.io/crates/d/apollo-parser.svg?style=flat-square" alt="Download" />
    </a>
    <a href="https://docs.rs/apollo-parser/">
        <img src="https://img.shields.io/static/v1?label=docs&message=apollo-parser&color=blue&style=flat-square" alt="docs.rs docs" />
    </a>
  </p>
</div>

## Features
* Typed GraphQL AST as per [October 2021 specification]
* Error resilience
  * lexing and parsing does not fail or `panic` if a lexical or a syntax error is found
* GraphQL lexer
* GraphQL parser

## Getting started
Add this to your `Cargo.toml` to start using `apollo-parser`:
```toml
# Just an example, change to the necessary package version.
[dependencies]
apollo-parser = "0.3.2"
```

Or using [cargo-edit]:
```bash
cargo add apollo-parser
```

## Usage
`apollo-parser` is built to parse both GraphQL schemas and queries according to
the latest [October 2021 specification]. It produces a typed syntax tree that
then can be walked, extracting all the necessary information. You can quick
start with:

```rust
use apollo_parser::Parser;

let input = "union SearchResult = Photo | Person | Cat | Dog";
let parser = Parser::new(input);
let ast = parser.parse();
```

`apollo-parser` is built to be error-resilient. This means we don't abort parsing (or lexing) if an error occurs. That means `parser.parse()` will always produce an AST, and it will be accompanied by any errors that are encountered:

```rust
use apollo_parser::Parser;

let input = "union SearchResult = Photo | Person | Cat | Dog";
let parser = Parser::new(input);
let ast = parser.parse();

// ast.errors() returns an iterator with the errors encountered during lexing and parsing
assert_eq!(0, ast.errors().len());

// ast.document() gets the Document, or root node, of the tree that you can
// start iterating on.
let doc = ast.document();
```

### Examples

Two examples outlined here:
* [Get field names in an object]
* [Get variables used in a query]

The [examples directory] in this repository has a few more useful
implementations such as:
* [using apollo-rs with miette to display error diagnostics]
* [using apollo-rs with annotate_snippets to display error diagnostics]
* [checking for unused variables]

#### Get field names in an object

```rust
use apollo_parser::{ast, Parser};

let input = "
type ProductDimension {
  size: String
  weight: Float @tag(name: \"hi from inventory value type field\")
}
";
let parser = Parser::new(input);
let ast = parser.parse();
assert_eq!(0, ast.errors().len());

let doc = ast.document();

for def in doc.definitions() {
    if let ast::Definition::ObjectTypeDefinition(object_type) = def {
        assert_eq!(object_type.name().unwrap().text(), "ProductDimension");
        for field_def in object_type.fields_definition().unwrap().field_definitions() {
            println!("{}", field_def.name().unwrap().text()); // size weight
        }
    }
}
```

#### Get variables used in a query

```rust
use apollo_parser::{ast, Parser};

let input = "
  query GraphQuery($graph_id: ID!, $variant: String) {
    service(id: $graph_id) {
      schema(tag: $variant) {
        document
      }
    }
  }
  ";

  let parser = Parser::new(input);
  let ast = parser.parse();
  assert_eq!(0, ast.errors().len());

  let doc = ast.document();

  for def in doc.definitions() {
      if let ast::Definition::OperationDefinition(op_def) = def {
          assert_eq!(op_def.name().unwrap().text(), "GraphQuery");

          let variable_defs = op_def.variable_definitions();
          let variables: Vec<String> = variable_defs
              .iter()
              .map(|v| v.variable_definitions())
              .flatten()
              .filter_map(|v| Some(v.variable()?.text().to_string()))
              .collect();
          assert_eq!(
              variables.as_slice(),
              ["graph_id".to_string(), "variant".to_string()]
          );
      }
  }
```

## License
Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or https://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or https://opensource.org/licenses/MIT)

at your option.

[cargo-edit]: https://github.com/killercup/cargo-edit
[apollo-rs: spec-compliant GraphQL Tools in Rust]: https://www.apollographql.com/blog/announcement/tooling/apollo-rs-graphql-tools-in-rust/
[examples directory]: https://github.com/apollographql/apollo-rs/tree/main/crates/apollo-parser/examples
[Get field names in an object]: https://github.com/apollographql/apollo-rs#get-field-names-in-an-object
[Get variables used in a query]: https://github.com/apollographql/apollo-rs#get-variables-used-in-a-query
[using apollo-rs with miette to display error diagnostics]: https://github.com/apollographql/apollo-rs/blob/a7f616454a53dcb8496725ceac6c63eacddefb2c/crates/apollo-parser/examples/miette.rs
[using apollo-rs with annotate_snippets to display error diagnostics]: https://github.com/apollographql/apollo-rs/blob/a7f616454a53dcb8496725ceac6c63eacddefb2c/crates/apollo-parser/examples/annotate_snippet.rs
[checking for unused variables]: https://github.com/apollographql/apollo-rs/blob/a7f616454a53dcb8496725ceac6c63eacddefb2c/crates/apollo-parser/examples/unused_vars.rs
[October 2021 specification]: https://spec.graphql.org/October2021
