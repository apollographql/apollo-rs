# Architecture

This document gives an overview of how various bits of `apollo-rs` work together (and separately). `apollo-rs` is intended to be a workspace of several crates to encapsulate generic GraphQL tooling in Rust. Currently this houses [`apollo-parser`] and [`apollo-encoder`].

## `apollo-parser`
`apollo-parser` is a recursive-descent spec-compliant parser of the GraphQL query language. Upon parsing a schema or a query it creates a typed AST. It consists of a hand-rolled lexer, parser and a generated API that helps query a tree. The overarching design is influenced by [`rust-analyzer`], and uses a few of its adjacent crates, like [`rowan`] and [`ungrammar`].

### Overview
![An overview of apollo-parser diagram. We initially start of with input data. A Lexer performs lexical analysis on the data and produces tokens. Tokens get syntactically analysed by the parser, first into an untyped syntax tree, then into a typed syntax tree. "Future Work" indicates that a typed syntax tree will be semantically analysed by the "Analyser" to produce a semantic model.](apollo-parser-overview.png)

### Lexer
TODO!

### Parser
TODO!

### AST
TODO!


[`apollo-parser`]: https://github.com/apollographql/apollo-rs/tree/main/crates/apollo-parser
[`apollo-encoder`]: https://github.com/apollographql/apollo-rs/tree/main/crates/apollo-encoder
[`rust-analyzer`]: https://github.com/rust-analyzer/rust-analyzer
[`rowan`]: https://github.com/rust-analyzer/rowan
[`ungrammar`]: https://github.com/rust-analyzer/ungrammar