<div align="center">
  <h1><code>apollo-rs</code></h1>

  <p>
    <strong>Rust tooling for low-level manipulation of the GraphQL language.</strong>
  </p>
</div>

## Tools included

This project is intended to house a number of tools related to the low-level
workings of GraphQL according to the [GraphQL specification]. Nothing in
these libraries is specific to Apollo, and can freely be used by other
projects which need standards-compliant GraphQL tooling written in Rust. The
following crates currently exist:

* [**`apollo-encoder`**](crates/apollo-encoder) - a library to generate GraphQL code (SDL).
* [**`apollo-parser`**](crates/apollo-parser) - a library to parse the GraphQL query language.
* [**`apollo-smith`**](crates/apollo-smith) - a test case generator to test GraphQL code (SDL).

Please check out their respective READMEs for usage examples.

## Status
`apollo-rs` is a work in progress. Please check out the
[ROADMAP](ROADMAP.md) for upcoming features we are working on building.

If you do end up trying out `apollo-rs` and run into trouble, we encourage you 
to open an [issue].

## Design Principles
1. **Prioritizing developer experience.** Elegant and ergonomic APIs is the
theme for Rust as a language, and we want to make sure that all component APIs
we provide are aligned with these principles.

2. **Stability and reliability.** Spec-compliant, and idempotent APIs which,
when complete, can be used safely in enterprise-grade codebases.

3. **Diagnostics.** The tools are to be written in a way that will allow us to
produce detailed diagnostics. It does not panic or return early if there is a
lexical or a syntactic error. Instead, the parser is meant to gather as much
context and information as possible and return errors alongside the output that
is valid. Coincidentally, this allows for easily debuggable code for those
maintaining this project.

4. **Extensibility.** The parser is written to work with different use cases in
our budding Rust GraphQL ecosystem, be it building schema-diagnostics for Rover,
or writing out query planning and composition algorithms in Rust. These all have
quite different requirements when it comes to AST manipulation. We wanted to
make sure we account for them early on.

## License
Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or https://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or https://opensource.org/licenses/MIT)

at your option.

[issue]: https://github.com/apollographql/apollo-rs/issues/new/choose
[GraphQL specification]: https://spec.graphql.org/October2021