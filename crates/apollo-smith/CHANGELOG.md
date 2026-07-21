
# Changelog

All notable changes to `apollo-smith` will be documented in this file.

This project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

<!-- # [x.x.x] (unreleased) - 2023-mm-dd

> Important: X breaking changes below, indicated by **BREAKING**

## BREAKING

## Features

## Fixes

## Maintenance

## Documentation -->
# [0.16.0](https://crates.io/crates/apollo-smith/0.16.0) - 2026-07-21

Important: 6 breaking changes below, indicated by **BREAKING**

## BREAKING

- **Make response generation configurable - [SharkBaitDLS], [tninesling], [pull/1033]**

  `ResponseBuilder` now supports `with_min_list_size`, `with_max_list_size`, and
  `with_null_ratio` to control the shape of generated responses, plus `with_generator`
  to register a custom `Generator` for any scalar, object, interface, or union
  type by name. It can also be driven by either `arbitrary::Unstructured` or a
  standard `rand::Rng` (via the new `RandProvider` wrapper), so the same builder
  works for both fuzz testing and general-purpose mock data. The primary breaking
  change is that `ResponseBuilder` is now generic over its randomness source,
  and its error type changed to `ResponseError`.

- **Improve byte efficiency for type and field name generation - [tninesling], [pull/1040]**

  This change makes document generation more efficient by using `Unstructured::choose`
  when building type names from characters, instead of generating a random
  `usize` and indexing into the character set. This mitigates cases where the
  generator would consume all bytes in the sequence before finishing the
  document, resulting in only one instance of each type. There is no breaking
  change to the API, but it does change the name selections chosen for types and
  therefore changes the generated documents.

- **Pass the correct directive location to input_values_def - [tninesling], [pull/1053]**

  This change adds a new directive location parameter to `DocumentBuilder::input_values_def`.
  This specifying whether it is generating an input value definition for an
  argument or an input object field. This is necessary to ensure that the
  correct directives are applied to the generated input value definition.

- **Make upper bounds for type counts configurable - [tninesling], [pull/1054]**

  Previously, the document generator would create up to 50 instances of each type kind. Now,
  you can specify the upper bound for how many of each type are generated.

  ```rust
  let builder = DocumentBuilder::new().max_scalar_types(75);
  ```

  The breaking change is that the constructor now infallibly returns a builder
  instead of a partial document, which allows chaining these type count bounds
  before document generation. In order to provide a more standardized API, the
  previous API's `.finish()` method is replaced with the more usual `.build()`.

  **Before**

  ```rust
  let mut u = Unstructured::new(fuzzer_input);
  let gql_doc = DocumentBuilder::new(&mut u)?;
  let doc = gql_doc.finish();
  ```

  **After**

  ```rust
  let mut u = Unstructured::new(fuzzer_input);
  let builder = DocumentBuilder::new(&mut u);
  let doc = builder.build()?;
  ```

- **Prevent extensions from duplicating existing values - [tninesling], [pull/1057]**

  Adds a new `exclude` parameter to `DocumentBuilder::enum_values_definition` and
  `DocumentBuilder::input_values_def`, plus the argument of the same name in
  `DocumentBuilder::fields_definition` is now an `IndexSet` instead of `&[&Name]`
  to match the other two.

  When generating the initial definition for any of these, an empty `IndexSet`
  should be passed. They are used internally when generating extensions to
  to existing types, ensuring that an extension does not duplicate some
  already-chosen field or value from the base definition.

- **Prevent non-null self-references in input objects - [tninesling], [pull/1062]**

  The `DocumentBuilder::input_values_def` function now takes a `self_name: Option<&Name>`
  parameter so inner value definitions do not choose a non-null value of the type
  being generated, which would otherwise cause an impossible cycle.

## Fixes

- **Ensure type names are unique across all type kinds - [tninesling], [pull/1045], [pull/1058], and [pull/1063]**
- **Choose input definition types from valid input types - [tninesling], [pull/1046]**
- **Ensure scalar and union extensions correctly extend original types - [tninesling], [pull/1047]**
- **Ensure generated documents do not have unused fragment definitions - [tninesling], [pull/1048]**
- **Allow directive applications when only one directive is defined - [duckki], [pull/1049]**
- **Generate valid interface inheritance - [tninesling], [pull/1050]**
- **Produce schemas with valid root operation types - [tninesling], [pull/1051]**
- **Restrict union members to object types - [tninesling], [pull/1052]**
- **Disable alias generation to avoid field selection merging conflicts - [tninesling], [pull/1059]**
- **Fix fragment spread and argument selection merging - [tninesling], [pull/1060]**

## Maintenance

- **update dependency rust to v2 - [pull/1055]**
- **update dependency gh to v3 - [pull/1056]**
- **update rust crate anyhow to 1.0.103 - [pull/1064]**

[duckki]: https://github.com/duckki
[tninesling]: https://github.com/tninesling
[SharkBaitDLS]: https://github.com/SharkBaitDLS
[pull/1064]: https://github.com/apollographql/apollo-rs/pull/1064
[pull/1063]: https://github.com/apollographql/apollo-rs/pull/1063
[pull/1062]: https://github.com/apollographql/apollo-rs/pull/1062
[pull/1060]: https://github.com/apollographql/apollo-rs/pull/1060
[pull/1059]: https://github.com/apollographql/apollo-rs/pull/1059
[pull/1058]: https://github.com/apollographql/apollo-rs/pull/1058
[pull/1057]: https://github.com/apollographql/apollo-rs/pull/1057
[pull/1056]: https://github.com/apollographql/apollo-rs/pull/1056
[pull/1055]: https://github.com/apollographql/apollo-rs/pull/1055
[pull/1054]: https://github.com/apollographql/apollo-rs/pull/1054
[pull/1053]: https://github.com/apollographql/apollo-rs/pull/1053
[pull/1052]: https://github.com/apollographql/apollo-rs/pull/1052
[pull/1051]: https://github.com/apollographql/apollo-rs/pull/1051
[pull/1050]: https://github.com/apollographql/apollo-rs/pull/1050
[pull/1049]: https://github.com/apollographql/apollo-rs/pull/1049
[pull/1048]: https://github.com/apollographql/apollo-rs/pull/1048
[pull/1047]: https://github.com/apollographql/apollo-rs/pull/1047
[pull/1046]: https://github.com/apollographql/apollo-rs/pull/1046
[pull/1045]: https://github.com/apollographql/apollo-rs/pull/1045
[pull/1040]: https://github.com/apollographql/apollo-rs/pull/1040
[pull/1033]: https://github.com/apollographql/apollo-rs/pull/1033

# [0.15.2](https://crates.io/crates/apollo-smith/0.15.2) - 2025-11-10

## Fixes
- **Return arbitrary::Error::IncorrectFormat for unsupported floats- [tninesling], [pull/1005]**
  When generating floats for GraphQL documents, we were naively unwrapping the
  conversion from `f64` to `serde_json::Number`. This would panic when
  `arbitrary` returned `f64::INFINITY` of `f64::NAN` because the `Number`
  conversion only works when its input is finite. In this case, the underlying
  bytes used to generate the value are considered to be in an invalid format.
  So, we return `arbitrary::Error::IncorrectFormat` to tell fuzzers to use a
  different seed in the future.

[tninesling]: https://github.com/tninesling
[pull/1005]: https://github.com/apollographql/apollo-rs/pull/1005

## Maintenance
- **Apply new clippy rules from Rust 1.90 - [goto-bus-stop], [pull/1001]**

[goto-bus-stop]: https://github.com/goto-bus-stop
[pull/1001]: https://github.com/apollographql/apollo-rs/pull/1001

- **bump minimum arbitrary version - [goto-bus-stop], [pull/1007]**

[goto-bus-stop]: https://github.com/goto-bus-stop
[pull/1007]: https://github.com/apollographql/apollo-rs/pull/1007

# [0.15.1](https://crates.io/crates/apollo-smith/0.15.1) - 2025-08-08

- **Implement builder for arbitrary responses - [tninesling], [pull/981]**

[tninesling]: https://github.com/tninesling
[pull/981]: https://github.com/apollographql/apollo-rs/pull/981


# [0.15.0](https://crates.io/crates/apollo-smith/0.14.0) - 2025-01-16

- **Update apollo-compiler dependency to stable `^1.25.0`**

# [0.14.0](https://crates.io/crates/apollo-smith/0.14.0) - 2024-09-24

- **Update apollo-compiler dependency to `=1.0.0-beta.24`**

# [0.13.0](https://crates.io/crates/apollo-smith/0.13.0) - 2024-09-17

- **Update apollo-compiler dependency to `=1.0.0-beta.23`**

# [0.12.0](https://crates.io/crates/apollo-smith/0.12.0) - 2024-09-09

- **Update apollo-compiler dependency to `=1.0.0-beta.22`**

# [0.11.0](https://crates.io/crates/apollo-smith/0.11.0) - 2024-09-03

- **Update apollo-compiler dependency to `=1.0.0-beta.21`**

# [0.10.0](https://crates.io/crates/apollo-smith/0.10.0) - 2024-07-31

- **Update apollo-parser dependency to `0.8.0`**
- **Update apollo-compiler dependency to `=1.0.0-beta.20`**

# [0.9.0](https://crates.io/crates/apollo-smith/0.9.0) - 2024-07-19

- **Update apollo-compiler dependency to `=1.0.0-beta.19`**

# [0.8.0](https://crates.io/crates/apollo-smith/0.8.0) - 2024-06-27

- **Update apollo-compiler dependency to `=1.0.0-beta.18`**

# [0.7.0](https://crates.io/crates/apollo-smith/0.7.0) - 2024-06-20

## Features

- **Improve field variability in Selection Set generation - [geal], [pull/866].**
  This changes the field generation algorithm in apollo-smith to allow more
  variety, because the previous implementation was the previous implementation
  was too biased towards the first field specified in a type. This also adds an
  example to generate a random query from a schema.

[geal]: https://github.com/geal
[pull/866]: https://github.com/apollographql/apollo-rs/pull/866


# [0.6.0-beta.1](https://crates.io/crates/apollo-smith/0.6.0-beta.1) - 2023-11-30

## BREAKING

- **Remove the `parser-impl` feature flag - [SimonSapin], [pull/754].**
  This functionality is now always enabled.
- **Use apollo-compiler instead of apollo-encoder for serialization - [SimonSapin], [pull/754].**
  The exact string output may change.

## Fixes

- **Make serialization ordering deterministic - [SimonSapin], [pull/754].**
  Internally use `IndexMap` and `IndexSet` instead of `IndexMap` and `IndexSet`

[SimonSapin]: https://github.com/SimonSapin
[pull/754]: https://github.com/apollographql/apollo-rs/pull/754

# [0.5.0](https://crates.io/crates/apollo-smith/0.5.0) - 2023-10-19

## BREAKING
- **apollo-parser@0.7.0 - [SimonSapin], [pull/694]**

  This updates the version of `apollo-parser` required by the `TryFrom`
  implementations in this crate.

- **removes `tryfrom` from apollo-compiler - [SimonSapin]**

  `apollo-compiler@1.0.0` can be directly serialised to sdl without requiring
  apollo-encoder. the `tryfrom` implementation is therefore no longer necessary.

[SimonSapin]: https://github.com/SimonSapin
[pull/694]: https://github.com/apollographql/apollo-rs/pull/694

# [0.4.0](https://crates.io/crates/apollo-smith/0.4.0) - 2023-08-21

## BREAKING
- **apollo-parser@0.6.0 - [goto-bus-stop], [pull/621]**

  This updates the version of `apollo-parser` required by the `TryFrom`
  implementations in this crate.

- **apollo-encoder@0.7.0 - [goto-bus-stop], [pull/623]**

  This updates the version of `apollo-encoder` required by the `From`
  implementations in this crate.

[goto-bus-stop]: https://github.com/goto-bus-stop
[pull/621]: https://github.com/apollographql/apollo-rs/pull/621
[pull/623]: https://github.com/apollographql/apollo-rs/pull/623

# [0.3.2](https://crates.io/crates/apollo-smith/0.3.2) - 2023-01-18
## Features
- Derive `Clone` on `apollo-smith` types, [SimonSapin] in [429]
- Add `DocumentBuilder::input_exhausted`, [SimonSapin] in [430]
## Fixes
- TryFrom for enums to use std::Result, continuation of [390], [bnjjj] in [428]
- Break infinite loop in input string generation, [SimonSapin] in [427]

[SimonSapin]: https://github.com/SimonSapin
[bnjjj]: https://github.com/bnjjj
[427]: https://github.com/apollographql/apollo-rs/pull/427
[428]: https://github.com/apollographql/apollo-rs/pull/428
[429]: https://github.com/apollographql/apollo-rs/pull/429
[430]: https://github.com/apollographql/apollo-rs/pull/430

# [0.3.1](https://crates.io/crates/apollo-smith/0.3.1) - 2022-11-29
This is a re-publish of 0.3.0 with fixed dependency versions.

# [0.3.0](https://crates.io/crates/apollo-smith/0.3.0) - 2022-11-29 (YANKED)

## BREAKING
- **make conversions from apollo-parser types fallible - [goto-bus-stop], [pull/371]**

  The `parser-impl` feature flag contains conversion code from apollo-parser AST node types
  to apollo-smith types. With this change, those conversions now use the `TryFrom` trait
  instead of the `From` trait, and return errors instead of panicking.

  You now have to use the `try_from()` and `try_into()` methods instead of `from()` and
  `into()`.

  [goto-bus-stop]: https://github.com/goto-bus-stop
  [pull/371]: https://github.com/apollographql/apollo-rs/pull/371

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
