<div align="center">
  <h1><code>apollo-rs</code></h1>

  <p>
    <strong>Rust tooling for low-level manipulation of the GraphQL language.</strong>
  </p>
</div>

# Tools included

This project is intended to house a number of tools related to the low-level
workings of GraphQL according to the [GraphQL
specification (June 2018)](https://spec.graphql.org/June2018). Nothing in
these libraries is specific to Apollo, and can freely be used by other
projects which need standards-compliant GraphQL tooling written in Rust. The
following crates currently exist:

* [**`apollo-encoder`**](apollo-encoder) - a library to generate GraphQL code.
* [**`apollo-parser`**](apollo-parser) - a library to parse the GraphQL
  query language.

# License

This project is licensed under the Apache 2.0 license.
See [LICENSE](LICENSE) for more details.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this project by you, as defined in the Apache-2.0 license,
shall be licensed as above, without any additional terms or conditions.