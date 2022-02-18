
# Changelog

All notable changes to `apollo-smith` will be documented in this file.

This project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

<!-- # [x.x.x] (unreleased) - 2021-mm-dd

> Important: X breaking changes below, indicated by **BREAKING**

## BREAKING

## Features

## Fixes

## Maintenance

## Documentation -->

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