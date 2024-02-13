# Architecture

This document gives an overview of how various bits of `apollo-rs` work together (and separately). `apollo-rs` is intended to be a workspace of several crates to encapsulate generic GraphQL tooling in Rust. Currently this houses [`apollo-parser`], [`apollo-compiler`] and [`apollo-smith`].

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
quite different requirements when it comes to CST manipulation. We wanted to
make sure we account for them early on.

## `apollo-parser`

`apollo-parser` is the parser crate of `apollo-rs`. Its job is to take GraphQL
queries or schemas as input and produce an Concrete Syntax Tree (CST). Users of
apollo-parser can then programmatically traverse the CST to get information
about their input. 

There are three main components of `apollo-parser`: the lexer, the parser, and the analyser. We already have the lexer and the parser, and the analyser is in the process of getting written.

![An overview of apollo-parser diagram. We initially start of with input data. A Lexer performs lexical analysis on the data and produces tokens. Tokens get syntactically analysed by the parser, first into an untyped syntax tree, then into a typed syntax tree. "Future Work" indicates that a typed syntax tree will be semantically analysed by the "Analyser" to produce a semantic model.](images/apollo-parser-overview.png)

`apollo-parser` is a hand-written recursive-descent parser. This is a type of
parser that starts from the top of a file and recursively walks its way down
generating CST nodes along the way. This style of parser is common in
industrial-strength compilers; for example, Clang and Rustc use this style of
parsing. In particular, recursive-descent parsers make it easier to output
helpful diagnostics. They perform well and they’re easier to maintain.

The overarching design is influenced by [`rust-analyzer`], and uses a few
of its adjacent crates, like [`rowan`] and [`ungrammar`].

### Lexer
The lexer takes input GraphQL and produces tokens based on the input. It
provides the guarantee that all tokens are **lexically correct**. The tokens are
then passed to the parser.

The lexer is designed to be error resilient, meaning the lexing step _never fails_.
When encountering an error during lexing, instead of exiting early and returning
only the error, we return all valid tokens alongside the error that occurs. The error will usually have the entire incorrect token data that it gathers as it loops over all the characters. Here is, for example, how we return either a correct Int or Float token, or an Error with the incorrect token data when creating numbers in the lexer. The lexer then continues lexing the remaining tokens in the input.

```rust
if let Some(mut err) = self.err() {
    err.data = buf;
    return Err(err);
}

if has_exponent || has_fractional {
    Ok(Token::new(TokenKind::Float, buf))
} else {
    Ok(Token::new(TokenKind::Int, buf))
}
```

The lexer returns a `Vec<Token>` and `Vec<Error>` that the parser then uses to create a tree. If the error vec is empty, we can be sure that the input is lexically correct!

### Parser
The next step in our parsing pipeline is the parser. The parser’s job is to take the tokens produced by the lexer and create nodes with information and relationships that in the end make up a syntax tree. Much like with the lexer, the parser is error resilient. Syntactic errors, such as a missing `Name` in a `ScalarDefinition`, are added to parser’s error vector while the parser carries on parsing.

![A diagram of how lexer’s tokens are arranged by the parser. On the left, different coloured boxes are stacked on top of on top of each other. These boxes represent various tokens created by the lexer. To the right, the boxes are rearranged in an upside down “tree” structure. The top of the tree is a single node. the boxes are arranged underneath the node in a top-down, left-to-right order they appear on the left. This is meant to represent the fact that the parser groups various tokens together and establishes relationships between them.](images/apollo_parser_tree_manipulation.png)

The parsing step is done in two parts: we first create an untyped syntax tree, then a typed one. 

#### Untyped Syntax Tree
We first create an untyped syntax tree when we manually parse incoming tokens.
This tree is stored with the help of [`rowan`] crate, a really quite excellent
library written by the rust-analyzer team. rowan creates a [Red/Green tree],
which is an efficient way of representing CSTs that can be updated over time.
This is a common technique used in many modern compilers such as Rust-Analyzer
and the Swift compiler.

The untyped tree stores information about the nodes, such as the token’s data and its relationship to other tokens in the tree, but not Rust type data; that comes later. We build the tree as we walk down the list of tokens. This is, for example, how we build the tree for a `ScalarTypeDefinition`:

```graphql
# schema.graphql
scalar UUID @specifiedBy(url: "cats.com/cool-kitten-schema")
```

The parser for the scalar is built something like this:
```rust
// grammar/scalar.rs

/// See: https://spec.graphql.org/October2021/#ScalarTypeDefinition
///
/// ScalarTypeDefinition =
///   Description? 'scalar' Name Directives?
pub(crate) fn scalar_type_definition(parser: &mut Parser) {

    // We already know this is a Scalar Type, so we go ahead and start a
    // SCALAR_TYPE_DEFINITION node.
    // 
    // This is not yet an actual Rust type, but a simple enum that later gets
    // converted to a Rust type.
    let _guard = parser.start_node(SyntaxKind::SCALAR_TYPE_DEFINITION);

    // Descriptions are optional, so we just check whether or not the lexer provided
    // us with a token that represents a description and add it to the node we
    // started above.
    if let Some(TokenKind::StringValue) = parser.peek() {
        description::description(parser);
    }
    
    // Add the "scalar" keyword to the node.
    if let Some("scalar") = parser.peek_data().as_deref() {
        parser.bump(SyntaxKind::scalar_KW);
    }

    // A Scalar Type must have a Name. If it doesn't have a Name token, we add
    // an error to our parser's error vector and don't add anything to the node.
    match parser.peek() {
        Some(TokenKind::Name) => name::name(parser),
        _ => parser.err("expected a Name"),
    }

    // Finally, we check if a directive was provided and add it to the current node.
    if let Some(TokenKind![@]) = parser.peek() {
        directive::directives(parser);
    }
    
    // This is the end of the ScalarTypeDefinition parsing and the
    // SCALAR_TYPE_DEFINITION node automatically gets closed and added to the
    // current untyped syntax tree.
}
```

#### Parser’s Typed Syntax Tree

Once the incoming token stream is done parsing, we create a typed syntax tree,
which is the basis of the parser’s API.

The accessor methods to the typed tree are generated using [`ungrammar`]
crate, another great Rust-Analyzer Team Original(TM). Ungrammar is a
domain-specific language (DSL) that allows us to specify the shape of our syntax
tree. If you’re interested to learn more about this crate’s design, you can read
about it in [this post].

So, how do we actually specify what our syntax tree should look like? Here is a
small example of how we define `ScalarTypeDefinition`
```ungram
// graphql.ungram

ScalarTypeDefinition =
  Description? 'scalar' Name Directives?
```

Our entire tree is defined in [graphql.ungram]. Using the `ungrammar`
definitions, we then create an entire `nodes.rs` file, like the snippet below,
using [`xtask`] in [codegen/gen_syntax_nodes]. We also generate
`SyntaxKind`([codegen/gen_syntax_kinds]) that we use when creating nodes in the
untyped syntax tree. 

Given the definition, we can then generate a struct with applicable accessor methods for `Description`, 
scalar token, `Name` and `Directives`.

```rust
// generated nodes.rs file

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ScalarTypeDefinition {
    pub(crate) syntax: SyntaxNode,
}

impl ScalarTypeDefinition {
    pub fn description(&self) -> Option<Description> {
        support::child(&self.syntax)
    }
    pub fn scalar_token(&self) -> Option<SyntaxToken> {
        support::token(&self.syntax, S![scalar])
    }
    pub fn name(&self) -> Option<Name> {
        support::child(&self.syntax)
    }
    pub fn directives(&self) -> Option<Directives> {
        support::child(&self.syntax)
    }
}
```


Here is what it looks like to walk a `ScalarDefinition` and get its `Name` from
the syntax tree using the above `name()` getter method:

```rust
use apollo_parser::Parser;
use apollo_parser::cst::{Definition, ObjectTypeDefinition};

// Let's create a GraphQL document with just a scalar type definition.
let gql = r#"scalar UUID @specifiedBy(url: "cats.com/cool-kitten-schema")"#;

// Parse the input data.
let parser = Parser::new(gql);
let cst = parser.parse();

// Make sure the are no errors.
assert!(cst.errors.is_empty());

// Check that the Scalar's name is indeed UUID.
let document = cst.document();
for definition in document.definitions() {
    if let Definition::ScalarTypeDefinition(scalar_type) = definition {
        assert_eq!("UUID", *scalar_type**.**name**()*.unwrap().text().to_string());
    }
}
```

[`apollo-parser`]: https://github.com/apollographql/apollo-rs/tree/main/crates/apollo-parser
[`apollo-compiler`]: https://github.com/apollographql/apollo-rs/tree/main/crates/apollo-compiler
[`apollo-smith`]: https://github.com/apollographql/apollo-rs/tree/main/crates/apollo-smith
[`rust-analyzer`]: https://github.com/rust-analyzer/rust-analyzer
[`rowan`]: https://github.com/rust-analyzer/rowan
[`ungrammar`]: https://github.com/rust-analyzer/ungrammar
[apollo-rs: spec-compliant GraphQL tools in Rust]: https://www.apollographql.com/blog/announcement/tooling/apollo-rs-graphql-tools-in-rust/
[Red/Green tree]: https://blog.yaakov.online/red-green-trees/
[this post]: https://rust-analyzer.github.io/blog/2020/10/24/introducing-ungrammar.html
[graphql.ungram]: https://github.com/apollographql/apollo-rs/blob/fcbf4903be261de1bf40756180f07cf339d3b2f9/graphql.ungram  
[`xtask`]: https://github.com/matklad/cargo-xtask
[codegen/gen_syntax_nodes]: https://github.com/apollographql/apollo-rs/blob/fcbf4903be261de1bf40756180f07cf339d3b2f9/xtask/src/codegen/gen_syntax_nodes.rs
[codegen/gen_syntax_kinds]: https://github.com/apollographql/apollo-rs/blob/fcbf4903be261de1bf40756180f07cf339d3b2f9/xtask/src/codegen/gen_syntax_kinds.rs