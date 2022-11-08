
# Changelog

All notable changes to `apollo-smith` will be documented in this file.

This project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

<!-- # [x.x.x] (unreleased) - 2022-mm-dd

> Important: X breaking changes below, indicated by **BREAKING**

## BREAKING

## Features

## Fixes

## Maintenance

## Documentation -->
# [0.2.0](https://crates.io/crates/apollo-smith/0.2.0) - 2022-11-08

## BREAKING

- **update apollo-parser@0.3.x - [lrlna], [pull/340], [pull/348]**

  This change was first released in the apollo-smith@0.1.4 patch release.
  It should have been a breaking change, as the update to the new version
  requires users to also update apollo-parser to 0.3.0 at the same time.

  This version is identical to 0.1.5 except for the version number.
  apollo-smith versions 0.1.4 and 0.1.5 have been yanked.

  [lrlna]: https://github.com/lrlna
  [pull/340]: https://github.com/apollographql/apollo-rs/pull/340
  [pull/348]: https://github.com/apollographql/apollo-rs/pull/348

# [0.1.5](https://crates.io/crates/apollo-smith/0.1.5) - 2022-11-04 (YANKED)

## Maintenance
- **update apollo-parser@0.3.1 - [lrlna], [pull/348]**

  [lrlna]: https://github.com/lrlna
  [pull/348]: https://github.com/apollographql/apollo-rs/pull/348

- **update apollo-encoder@0.3.4 - [lrlna], [pull/349]**

  [lrlna]: https://github.com/lrlna
  [pull/349]: https://github.com/apollographql/apollo-rs/pull/349

# [0.1.4](https://crates.io/crates/apollo-smith/0.1.4) - 2022-11-04 (YANKED)

## Maintenance
- **update apollo-parser@0.3.0 - [lrlna], [pull/340]**

  [lrlna]: https://github.com/lrlna
  [pull/340]: https://github.com/apollographql/apollo-rs/pull/340

# [0.1.3](https://crates.io/crates/apollo-smith/0.1.3) - 2022-05-12

## Fixes
- **add interface definition to internal stack - [bnjjj], [pull/213]**

  Added support of interface definition in the stack to fill an operation with
  correct fields.

  [bnjjj]: https://github.com/bnjjj
  [pull/213]: https://github.com/apollographql/apollo-rs/pull/213

# [0.1.2](https://crates.io/crates/apollo-smith/0.1.2) - 2022-04-28

## Maintenance
- **Update apollo-encoder to 0.3.0 - [lrlna], [pull/207] [pull/208]**
  `apollo-encoder`'s 0.3.0 changes `desciption` and `default-value` setters to
  accept String as a parameter. This changes the internals of apollo-smith
  accordingly.

  [lrlna]: https://github.com/lrlna
  [pull/207]: https://github.com/apollographql/apollo-rs/pull/207
  [pull/208]: https://github.com/apollographql/apollo-rs/pull/208

# [0.1.1](https://crates.io/crates/apollo-smith/0.1.1) - 2022-04-01
## Features
- **Add `parser-impl` feature flag - [bnjjj], [pull/197]**
  `parser-impl` feature in `apollo-smith` is used to convert
  `apollo-parser` types to `apollo-smith` types. This is useful when you require
  the test-case generator to generate documents based on a given schema.

  ```toml
  ## Cargo.toml

  [dependencies]
  apollo-smith = { version = "0.1.1", features = ["parser-impl"] }
  ```

  ```rust
  use std::fs;

  use apollo_parser::Parser;
  use apollo_smith::{Document, DocumentBuilder};

  use libfuzzer_sys::arbitrary::{Result, Unstructured};

  /// This generate an arbitrary valid GraphQL operation
  pub fn generate_valid_operation(input: &[u8]) {

      let parser = Parser::new(&fs::read_to_string("supergraph.graphql").expect("cannot read file"));

      let tree = parser.parse();
      if !tree.errors().is_empty() {
          panic!("cannot parse the graphql file");
      }

      let mut u = Unstructured::new(input);

      // Convert `apollo_parser::Document` into `apollo_smith::Document`.
      let apollo_smith_doc = Document::from(tree.document());

      // Create a `DocumentBuilder` given an existing document to match a schema.
      let mut gql_doc = DocumentBuilder::with_document(&mut u, apollo_smith_doc)?;
      let operation_def = gql_doc.operation_definition()?.unwrap();

      Ok(operation_def.into())
  }
  ```

  [bnjjj]: https://github.com/bnjjj
  [pull/197]: https://github.com/apollographql/apollo-rs/pull/197

- **Introduces semantic validations to the test-case generation - [bnjjj], [pull/197]**

  Semantic validations currently include:
    - Directives used in the document must already be defined
    - Directives must be unique in a given Directive Location
    - Default values must be of correct type
    - Input values must be of correct type
    - All type extensions are applied to an existing type
    - Field arguments in fragments and operation definitions must be defined on
      original type and must be of correct type

  [bnjjj]: https://github.com/bnjjj
  [pull/197]: https://github.com/apollographql/apollo-rs/pull/197

# [0.1.0](https://crates.io/crates/apollo-smith/0.1.0) - 2021-02-18

Introducing `apollo-smith`!

The goal of `apollo-smith` is to generate valid GraphQL documents by sampling
from all available possibilities of [GraphQL grammar].

We've written `apollo-smith` to use in fuzzing, but you may wish to use it for
anything that requires GraphQL document generation.

`apollo-smith` is inspired by bytecodealliance's [`wasm-smith`] crate, and the
[article written by Nick Fitzgerald] on writing test case generators in Rust.

This is still a work in progress, for outstanding issues, checkout out the
[apollo-smith label] in our issue tracker.

[GraphQL grammar]: https://spec.graphql.org/October2021/#sec-Appendix-Grammar-Summary
[`wasm-smith`]: https://github.com/bytecodealliance/wasm-tools/tree/main/crates/wasm-smith
[article written by Nick Fitzgerald]: https://fitzgeraldnick.com/2020/08/24/writing-a-test-case-generator.html#what-is-a-test-case-generator
[apollo-smith label]: https://github.com/apollographql/apollo-rs/labels/apollo-smith
