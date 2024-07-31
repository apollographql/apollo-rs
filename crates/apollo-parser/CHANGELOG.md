# Changelog

All notable changes to `apollo-parser` will be documented in this file.

This project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

<!-- # [x.x.x] (unreleased) - 2022-mm-dd

> Important: X breaking changes below, indicated by **BREAKING**
## BREAKING

## Features

## Fixes

## Maintenance

## Documentation -->

# [0.8.0](https://crates.io/crates/apollo-parser/0.8.0) - 2024-07-30

## BREAKING
This release removes the `Error::new` constructor. We recommend not creating instances of
`apollo_parser::Error` yourself at all.

## Fixes
- **add missing location information for lexer errors - [PhoebeSzmucer], [pull/886], [issue/731]**
  Unexpected characters now raise an error pointing to the character itself, instead of the start of the input document.

[PhoebeSzmucer]: https://github.com/PhoebeSzmucer
[issue/731]: https://github.com/apollographql/apollo-rs/issues/731
[pull/886]: https://github.com/apollographql/apollo-rs/pull/886

# [0.7.7](https://crates.io/crates/apollo-parser/0.7.7) - 2024-04-08

## Fixes
- **raise an error for empty field sets - [tinnou], [pull/845]**
  It's not legal to write `type Object {}` *with* braces but *without* declaring
  any fields. In the past this was accepted by apollo-parser, now it raises an
  error as required by the spec.

[tinnou]: https://github.com/tinnou
[pull/845]: https://github.com/apollographql/apollo-rs/pull/845

# [0.7.6](https://crates.io/crates/apollo-parser/0.7.6) - 2024-02-14

## Fixes
- **optimize the most common lexer matches into lookup tables - [allancalix], [pull/814]**
  Parsing large schema documents can be up to 18% faster, typical documents a few percent.
- **fix infinite loops and crashes found through fuzzing - [goto-bus-stop], [pull/828]**
  When using a token limit, it was possible to craft a document that would cause an infinite
  loop, eventually leading to an out of memory crash. This is addressed along with several panics.

## Maintenance
- **reduce intermediate string allocations - [goto-bus-stop], [pull/820]**

[allancalix]: https://github.com/allancalix
[goto-bus-stop]: https://github.com/goto-bus-stop
[pull/814]: https://github.com/apollographql/apollo-rs/pull/814
[pull/820]: https://github.com/apollographql/apollo-rs/pull/820
[pull/828]: https://github.com/apollographql/apollo-rs/pull/828

# [0.7.5](https://crates.io/crates/apollo-parser/0.7.5) - 2023-12-18

## Fixes
- **fix parsing `\\"""` in block string - [goto-bus-stop], [pull/774]**
  Previously this was parsed as `\` followed by the end of the string,
  now it's correctly parsed as `\` followed by an escaped `"""`.
- **emit syntax errors for variables in constant values - [SimonSapin], [pull/777]**
  default values and type system directive arguments are considered constants
  and may not use `$foo` variable values.
- **emit syntax errors for type condition without a type name - [rishabh3112], [pull/781]**

[goto-bus-stop]: https://github.com/goto-bus-stop
[SimonSapin]: https://github.com/SimonSapin
[rishabh3112]: https://github.com/rishabh3112
[pull/774]: https://github.com/apollographql/apollo-rs/pull/774
[pull/777]: https://github.com/apollographql/apollo-rs/pull/777
[pull/781]: https://github.com/apollographql/apollo-rs/pull/781

# [0.7.4](https://crates.io/crates/apollo-parser/0.7.4) - 2023-11-17

## Features
- **`parse_type` parses a selection set with optional outer brackets - [lrlna], [pull/718] fixing [issue/715]**
  This returns a `SyntaxTree<Type>` which instead of `.document() -> cst::Document`
  has `.type() -> cst::Type`.
  This is intended to parse the string value of a [`@field(type:)` argument][fieldtype]
  used in some Apollo Federation directives.
  ```rust
  let source = r#"[[NestedList!]]!"#;

  let parser = Parser::new(source);
  let cst: SyntaxTree<cst::Type> = parser.parse_type();
  let errors = cst.errors().collect::<Vec<_>>();
  assert_eq!(errors.len(), 0);
  ```

[lrlna]: https://github.com/lrlna
[pull/718]: https://github.com/apollographql/apollo-rs/pull/718
[issue/715]: https://github.com/apollographql/apollo-rs/issues/715
[fieldtype]: https://specs.apollo.dev/join/v0.3/#@field

## Fixes

- **Input object values can be empty - [goto-bus-stop], [pull/745] fixing [issue/744]**
  `apollo-parser` version 0.7.3 introduced a regression where empty input objects failed to parse.
  This is now fixed.

  ```graphql
  { field(argument: {}) }
  ```

[goto-bus-stop]: https://github.com/goto-bus-stop
[pull/745]: https://github.com/apollographql/apollo-rs/pull/745
[issue/744]: https://github.com/apollographql/apollo-rs/issues/744

# [0.7.3](https://crates.io/crates/apollo-parser/0.7.3) - 2023-11-07

## Fixes

- **Less recursion in parser implementation - [goto-bus-stop], [pull/721] fixing [issue/666]**
  The parser previously used recursive functions while parsing some repetitive nodes, like members of an enum:
  ```graphql
  enum Alphabet { A B C D E F G etc }
  ```
  Even though this is a flat list, each member would use a recursive call. Having many members, or fields in a type
  definition, or arguments in a directive, would all contribute to the recursion limit.

  Those cases are now using iteration instead and no longer contribute to the recursion limit. The default recursion limit
  is unchanged at 500, but you could reduce it depending on your needs.

[goto-bus-stop]: https://github.com/goto-bus-stop
[pull/721]: https://github.com/apollographql/apollo-rs/pull/721
[issue/666]: https://github.com/apollographql/apollo-rs/issues/666

- **Fix overly permissive parsing of `implements` lists and `union` member types - [goto-bus-stop], [pull/721] fixing [issue/659]**
  Previously these definitions were all accepted, despite missing or excessive `&` and `|` separators:
  ```graphql
  type Ty implements A B
  type Ty implements A && B
  type Ty implements A & B &

  union Ty = A B
  union Ty = A || B
  union Ty = A | B |
  ```
  Now they report a syntax error.

[goto-bus-stop]: https://github.com/goto-bus-stop
[pull/721]: https://github.com/apollographql/apollo-rs/pull/721
[issue/659]: https://github.com/apollographql/apollo-rs/issues/659

# [0.7.2](https://crates.io/crates/apollo-parser/0.7.2) - 2023-11-03

## Fixes

- **Fix `SyntaxTree` being accidentally `!Send` and `!Sync` - [SimonSapin], [pull/704] fixing [issue/702]**

[SimonSapin]: https://github.com/SimonSapin
[pull/704]: https://github.com/apollographql/apollo-rs/pull/704
[issue/702]: https://github.com/apollographql/apollo-rs/issues/702

# [0.7.1](https://crates.io/crates/apollo-parser/0.7.1) - 2023-10-10

## Features
- **`parse_field_set` parses a selection set with optional outer brackets - [lrlna], [pull/685] fixing [issue/681]**
  This returns a `SyntaxTree<SelectionSet>` which instead of `.document() -> cst::Document`
  has `.field_set() -> cst::SelectionSet`.
  This is intended to parse string value of a [`FieldSet` custom scalar][fieldset]
  used in some Apollo Federation directives.
  ```rust
  let source = r#"a { a }"#;

  let parser = Parser::new(source);
  let cst: SyntaxTree<cst::SelectionSet> = parser.parse_selection_set();
  let errors = cst.errors().collect::<Vec<_>>();
  assert_eq!(errors.len(), 0);
  ```

[lrlna]: https://github.com/lrlna
[pull/685]: https://github.com/apollographql/apollo-rs/pull/685
[issue/681]: https://github.com/apollographql/apollo-rs/issues/681
[fieldset]: https://www.apollographql.com/docs/federation/subgraph-spec/#scalar-fieldset


# [0.7.0](https://crates.io/crates/apollo-parser/0.7.0) - 2023-10-05

## BREAKING

- **rename `ast` to `cst` - [SimonSapin], [pull/???]**
  The Rowan-based typed syntax tree emitted by the parser used to be called
  Abstract Syntax Tree (AST) but is in fact not very abstract: it preserves
  text input losslessly, and all tree leaves are string-based tokens.
  This renames it to Concrete Syntax Tree (CST) and renames various APIs accordingly.
  This leaves the name available for a new AST in apollo-compiler 1.0.

# [0.6.3](https://crates.io/crates/apollo-parser/0.6.3) - 2023-10-06

## Fixes
- **apply recursion limit where needed, reduce its default from 4096 to 500 - [SimonSapin], [pull/662]**
  The limit was only tracked for nested selection sets, but the parser turns out
  to use recursion in other cases too. [Issue 666] tracks reducing them.
  Stack overflow was observed with little more than 2000
  nesting levels or repetitions in the new test.
  Defaulting to a quarter of that leaves a comfortable margin.
- **fix various lexer bugs - [SimonSapin], [pull/646], [pull/652]**
  The lexer was too permissive in emitting tokens instead of errors
  in various cases around numbers, strings, and EOF.
- **fix panic on surrogate code points in unicode escape sequences - [SimonSapin], [issue/608], [pull/658]**

[issue/608]: https://github.com/apollographql/apollo-rs/issues/608
[pull/646]: https://github.com/apollographql/apollo-rs/pull/646
[pull/652]: https://github.com/apollographql/apollo-rs/pull/652
[pull/658]: https://github.com/apollographql/apollo-rs/pull/658
[pull/662]: https://github.com/apollographql/apollo-rs/pull/662
[Issue 666]: https://github.com/apollographql/apollo-rs/issues/666

# [0.6.2](https://crates.io/crates/apollo-parser/0.6.2) - 2023-09-08
## Fixes
- **fixes to conversions from AST string nodes to Rust Strings - [goto-bus-stop], [pull/633], [issue/609], [issue/611]**
  This fix affects the `String::from(ast::StringValue)` conversion function, which returns the contents of a GraphQL string node.
  `"\""` was previously interpreted as just a backslash, now it is correctly interpreted as a double quote. For block strings, indentation is stripped as required by the spec.

[goto-bus-stop]: https://github.com/goto-bus-stop
[pull/633]: https://github.com/apollographql/apollo-rs/pull/633
[issue/609]: https://github.com/apollographql/apollo-rs/issues/609
[issue/611]: https://github.com/apollographql/apollo-rs/issues/611

# [0.6.1](https://crates.io/crates/apollo-parser/0.6.1) - 2023-08-28
## Fixes
- **fix lexing escape-sequence-like text in block strings - [goto-bus-stop], [pull/638], [issue/632]**
  Fixes a regression in 0.6.0 that could cause apollo-parser to reject valid input if a
  block string contained backslashes. Block strings do not support escape sequences so
  backslashes are normally literal, but 0.6.0 tried to lex them as escape sequences,
  which could be invalid (eg. `\W` is not a supported escape sequence).

  Now block strings are lexed like in 0.5.3. Only the `\"""` sequence is treated as an
  escape sequence.

[goto-bus-stop]: https://github.com/goto-bus-stop
[pull/638]: https://github.com/apollographql/apollo-rs/pull/638
[issue/632]: https://github.com/apollographql/apollo-rs/issues/632

# [0.6.0](https://crates.io/crates/apollo-parser/0.6.0) - 2023-08-18
## Features
- **zero-alloc lexer - [allancalix], [pull/322]**
  Rewrites the lexer to avoid allocating for each token. Synthetic benchmarks
  show about a 25% performance improvement to the parser as a whole.

## Fixes
- **fix token limit edge case - [goto-bus-stop], [pull/619], [issue/610]**
  `token_limit` now includes the EOF token. In the past you could get
  `token_limit + 1` tokens out of the lexer if the token at the limit was the
  EOF token, but now it really always stops at `token_limit`.

- **create EOF token with empty data - [allancalix], [pull/591]**
  Makes consuming the token stream's data produce an identical string to the
  original input of the lexer.

[allancalix]: https://github.com/allancalix
[goto-bus-stop]: https://github.com/goto-bus-stop
[issue/610]: https://github.com/apollographql/apollo-rs/issues/610
[pull/322]: https://github.com/apollographql/apollo-rs/pull/322
[pull/591]: https://github.com/apollographql/apollo-rs/pull/591
[pull/619]: https://github.com/apollographql/apollo-rs/pull/619

# [0.5.3](https://crates.io/crates/apollo-parser/0.5.3) - 2023-05-12
## Fixes
- **variable definition list cannot be empty - [lrlna], [pull/553] fixing [issue/546]**
  We previously allowed an operation with an empty variable definition list,
  which is incorrect. This change provides a fix.

[lrlna]: https://github.com/lrlna
[issue/546]: https://github.com/apollographql/apollo-rs/pull/546
[pull/553]: https://github.com/apollographql/apollo-rs/pull/553

# [0.5.2](https://crates.io/crates/apollo-parser/0.5.2) - 2023-05-10

## Features
- **add `SyntaxTree::token_limit` - [SimonSapin], [pull/525]**
  This enables finding out how many tokens were present in a succesful parse,
  which can be useful to choose where to set the limit.

- **add `Definition::kind() -> &str` and  `Definition::is_executable_definition()` - [goto-bus-stop], [pull/535]**
  These are new methods on the `Definition` AST node. `kind()` returns the kind
  of definition (eg. "ScalarTypeExtension") and `is_executable_definition()`
  returns true for operation definitions and fragment definitions.

[SimonSapin]: https://github.com/SimonSapin
[goto-bus-stop]: https://github.com/goto-bus-stop
[pull/525]: https://github.com/apollographql/apollo-rs/pull/525
[pull/535]: https://github.com/apollographql/apollo-rs/pull/535

## Fixes
- **handle escape sequences when reading string contents - [goto-bus-stop], [pull/541]**
  The `String::from(StringValue)` implementation now turns escape sequences like
  `\n` and `\u2764` into their literal characters.

[goto-bus-stop]: https://github.com/goto-bus-stop
[pull/541]: https://github.com/apollographql/apollo-rs/pull/541

# [0.5.1](https://crates.io/crates/apollo-parser/0.5.1) - 2023-04-13
## Fixes
- **remove recursion in field parsing - [goto-bus-stop], [pull/519]**
  The `selection::selection_set` parser already supports parsing multiple fields.
  This removes recursion from field parsing, reducing the risk of stack
  overflow on queries with many fields.

  [goto-bus-stop]: https://github.com/goto-bus-stop
  [pull/519]: https://github.com/apollographql/apollo-rs/pull/519


# [0.5.0](https://crates.io/crates/apollo-parser/0.5.0) - 2023-02-16
## Features
- **new `ast::Definition` methods - [goto-bus-stop], [pull/456]**
  When working with `Definition` nodes, you can use the `.name()` method to get the name of a definition, regardless of its kind. For `schema` definitions, it returns `None`.
  You can use `.is_extension_definition()` to check if a definition node is an extension.

  [goto-bus-stop]: https://github.com/goto-bus-stop
  [pull/456]: https://github.com/apollographql/apollo-rs/pull/456

## Fixes
- **fix token order around type names - [goto-bus-stop], [issue/362], [pull/443]**

  ```graphql
  type Query {
    field: Int # comment
  }
  ```
  Previously, the whitespace and comment around the `Int` type name would end up *before* the type name in the parse tree. This would mess up the location information for the `Int` type name. Now this is fixed.

  [goto-bus-stop]: https://github.com/goto-bus-stop
  [issue/362]: https://github.com/apollographql/apollo-rs/issues/362
  [pull/443]: https://github.com/apollographql/apollo-rs/pull/443

- **fix spans after parsing unexpected tokens - [goto-bus-stop], [issue/325], [pull/446]**

  Location information for all nodes after an unexpected token was incorrect. It's better now, though still imperfect: lexing errors still have this problem.

  [goto-bus-stop]: https://github.com/goto-bus-stop
  [issue/325]: https://github.com/apollographql/apollo-rs/issues/325
  [pull/446]: https://github.com/apollographql/apollo-rs/pull/446

- **fix ignored token positioning in the AST - [goto-bus-stop], [pull/445]**

  This makes spans for all nodes more specific, not including ignored tokens after the node. When ignored tokens are consumed, they are first stored separately, and then added to the AST just before the next node is started. This way ignored tokens are always inside the outermost possible node, and therefore all individual nodes will have spans that only contain that node and not more.

  The most obvious effect of this is that diagnostics now point to the exact thing they are about, instead of a bunch of whitespace :)

  [goto-bus-stop]: https://github.com/goto-bus-stop
  [pull/445]: https://github.com/apollographql/apollo-rs/pull/445

# [0.4.1](https://crates.io/crates/apollo-parser/0.4.1) - 2022-12-13
## Fixes
- **fix panics when parsing type names with syntax errors - [goto-bus-stop], [pull/381]**

  For example, `field: []` does not panic anymore. Instead it produces a syntax error and an incomplete List type.

- **continue parsing after a syntax error in an object type field - [goto-bus-stop], [pull/381]**

   ```graphql
   type A {
      fieldA: [] # ← has error, missing item type
      fieldB: Int
      fieldC: Int
   }
   ```
   Previously fieldB and fieldC would not be parsed, now they are.

  [goto-bus-stop]: https://github.com/goto-bus-stop
  [pull/381]: https://github.com/apollographql/apollo-rs/pull/381

# [0.4.0](https://crates.io/crates/apollo-parser/0.4.0) - 2022-11-28
## BREAKING
- **make conversions from GraphQL Values to Rust types fallible - [goto-bus-stop], [pull/371] fixing [issue/358]**

  In the past you could do:
  ```rust
  let graphql_value: IntValue = get_a_value();
  let x: i32 = graphql_value.into();
  ```
  But this `.into()` implementation could panic if the number was out of range.
  Now, this conversion is implemented with the `TryFrom` trait, so you handle out-of-range errors however you want:
  ```rust
  let graphql_value: IntValue = get_a_value();
  let x: i32 = graphql_value.try_into()?;
  ```

  [goto-bus-stop]: https://github.com/goto-bus-stop
  [pull/371]: https://github.com/apollographql/apollo-rs/pull/371
  [issue/358]: https://github.com/apollographql/apollo-rs/pull/358

- **Move `with_recursion_limit` constructor to a builder method - [goto-bus-stop], [pull/347]**

  If you were using the `Parser::with_recursion_limit` constructor, you now need to use `Parser::new().recursion_limit()` instead.

## Features
- **add API to limit number of tokens to parse - [goto-bus-stop], [pull/347]**

  When dealing with untrusted queries, malicious users can submit very large queries to attempt to cause
  denial-of-service by using lots of memory. To accompany the existing `recursion_limit` API preventing
  stack overflows, you can now use `token_limit` to abort parsing when a large number of tokens is reached.

  You can use the new `err.is_limit()` API to check if a parse failed because a hard limit was reached.

  ```rust
  let source = format!("query {{ {fields} }}", fields = "a ".repeat(20_000));

  let parser = Parser::new(source)
      .recursion_limit(10)
      // You may need an even higher limit if your application actually sends very large queries!
      .token_limit(10_000);

  let (ast, errors) = parser.parse();
  if errors.iter().any(|err| err.is_limit()) {
      // there was a limiting error
  }
  ```

  [goto-bus-stop]: https://github.com/goto-bus-stop
  [pull/347]: https://github.com/apollographql/apollo-rs/pull/347

## Maintenance
- **Use `eat()` in a loop instead of recursing in `bump()` - [goto-bus-stop], [pull/361]**

  [goto-bus-stop]: https://github.com/goto-bus-stop
  [pull/361]: https://github.com/apollographql/apollo-rs/pull/361

# [0.3.2](https://crates.io/crates/apollo-parser/0.3.2) - 2022-11-15
## Fixes
- **lexing escaped and unicode characters in block strings - [lrlna], [pull/357] fixing [issue/341], [issue/342], [issue/343]**

  Fixes lexing the following string values:
```graphql
"""unicode in block string 🤷"""
input Filter {
    title: String
}
"""
\""" a/b \"""
"""
input Filter {
    title: String
}

type Query {
    format: String = "Y-m-d\\TH:i:sP"
}
```

  [lrlna]: https://github.com/lrlna
  [pull/357]: https://github.com/apollographql/apollo-rs/pull/357
  [issue/341]: https://github.com/apollographql/apollo-rs/pull/341
  [issue/342]: https://github.com/apollographql/apollo-rs/pull/342
  [issue/343]: https://github.com/apollographql/apollo-rs/pull/343

# [0.3.1](https://crates.io/crates/apollo-parser/0.3.1) - 2022-11-04

## Features
- **streaming lexer - [Geal] + [goto-bus-stop], [pull/115]**

  To help improve performance and memory usage in the lexer, we are now
  streaming all incoming tokens in the lexer implementation.

  [goto-bus-stop]: https://github.com/goto-bus-stop
  [Geal]: https://github.com/Geal
  [pull/115]: https://github.com/apollographql/apollo-rs/pull/115

- **extend `ast::*Value` node conversions - [SimonSapin], [pull/344]**

  The following node types implement conversion to standard types, extracting
  their value:

  - `StringValue` → `String`
  - `IntValue` → `i32`
  - `FloatValue` → `f64`
  - `BoolValue` → `bool`

  These conversions are now also available:

  - Through the `From` trait, not just the `Into` trait
  - With borrowed nodes, not just owned

  Example:

  ```rust
  let node: &apollo_parser::ast::StringValue = /* something */;
  let value: String = node.clone().into(); // before
  let value = String::from(node); // now also possible
  ```

  [simonsapin]: https://github.com/SimonSapin
  [pull/344]: https://github.com/apollographql/apollo-rs/pull/344

## Documentation
- **example of modifying queries with parser + encoder - [lrlna], [pull/346]**
  An addition to `apollo-parser`'s [example] directory encoding various parts of the AST using `apollo-encoder`'s new `TryFrom` implementation. Examples include:

    - merging two queries
    - omitting certain fields in a query.

  [lrlna]: https://github.com/lrlna
  [pull/346]: https://github.com/apollographql/apollo-rs/pull/346
  [example]: ./examples/modify_query_using_parser_and_encoder.rs


# [0.3.0](https://crates.io/crates/apollo-parser/0.3.0) - 2022-10-31 💀
## BREAKING
- **remove the impl Display for generated nodes - [goto-bus-stop], [pull/330]**

  The `Display` impls for generated nodes returned the source text for that
  node. That's not a super common operation but it was very easy to access. It's
  also a very different operation from eg. `let content: String =
  node.string_value().into()` which returns the *content* of a string:
  `node.string_value().to_string()` returned the string as it was written in the
  source code, quotes and escapes and all.

  Now `.to_string()` is replaced by a `.source_string()` method. It allocates a
  new String (just like `.to_string()` did). A syntax node can represent
  multiple slices (I think to support different structures like Ropes as
  input?), so slicing the original source isn't actually possible.

  [goto-bus-stop]: https://github.com/goto-bus-stop
  [pull/330]: https://github.com/apollographql/apollo-rs/pull/330

## Fixes
- **handle unexpected tokens in top-level document parsing - [JrSchild], [pull/324]**
  Unexpected tokens directly inside a document would break the loop in the
  parser, for example:

  ```graphql
  @
  {
    name
  }}
  ```

  This resulted in the rest of the parsing to be skipped. An error is created
  here instead.

  [JrSchild]: https://github.com/JrSchild
  [pull/324]: https://github.com/apollographql/apollo-rs/pull/324

## Maintenance
- **reduce token copying - [goto-bus-stop], [pull/323]**


  * Reduce token copying

  Since the original lexer results are not needed anymore after this step,
  we can take ownership of the tokens and errors vectors and reverse them
  in-place without making a copy. Big schemas can have 100K+ tokens so
  it's actually quite a lot of work to copy them.

  * Reduce double-clones of tokens in the parser

  Some of these clones were not necessary. In particular the `.expect`
  method cloned the token unconditionally (including the string inside)
  and then cloned the string again immediately afterwards. This removes
  the first clone by reordering the `current.index()` call to satisfy the
  borrow checker.

  The `.data().to_string()` clone is only used in the error case, but
  avoiding that will require more work.

  [goto-bus-stop]: https://github.com/goto-bus-stop
  [pull/323]: https://github.com/apollographql/apollo-rs/pull/323

# [0.2.12](https://crates.io/crates/apollo-parser/0.2.12) - 2022-09-30
## Fixes
- **unterminated string values with line terminators and unicode- [lrlna], [pull/320] fixes [issue/318]**


  We were missing adding a line terminator character to the errors created by the lexer in case of a unterminated string. This showed up incidentally while dealing with unicode and the fact that it's of a different byte length than most other characters.

  [lrlna]: https://github.com/lrlna
  [pull/320]: https://github.com/apollographql/apollo-rs/pull/320
  [issue/318]: https://github.com/apollographql/apollo-rs/issues/318

# [0.2.11](https://crates.io/crates/apollo-parser/0.2.11) - 2022-09-20
## Features
- **introduce a getter to parser's green nodes - [lrlna], [pull/295]**

  creates a getter to parser's underlying green nodes that make up the
  resulting, more ergonomic AST. This is handy for our compiler's use case when
  creating a thread-safe access to the AST.

  [lrlna]: https://github.com/lrlna
  [pull/295]: https://github.com/apollographql/apollo-rs/pull/295

## Fixes
- **selection set is required for named operation definitions- [lrlna], [pull/301] closes [issue/300]**

  The parser was not creating errors for missing selection sets for named
  operation definitions such as `query namedQuery {`. This is now correctly
  flagged as erroneous graphql.

  [lrlna]: https://github.com/lrlna
  [pull/301]: https://github.com/apollographql/apollo-rs/pull/301
  [issue/300]: https://github.com/apollographql/apollo-rs/issues/300

# [0.2.10](https://crates.io/crates/apollo-parser/0.2.10) - 2022-08-16
## Fixes
- **unterminated string value in list and object values - [bnjjj], [pull/267] & [pull/274] closes [issue/266]**

  Create and pop errors with unterminated string values in list and object
  values. Stops infinite loop when searching for a Value in the parser.

  [bnjjj]: https://github.com/bnjjj
  [pull/267]: https://github.com/apollographql/apollo-rs/pull/267
  [issue/266]: https://github.com/apollographql/apollo-rs/issues/266


# [0.2.9](https://crates.io/crates/apollo-parser/0.2.9) - 2022-07-27
## Features
- **Provide APIs for SyntaxNode and SyntaxNodePtr - [lrlna], [pull/251]**

  Export a wrapper around SyntaxNodePtr provided by `rowan`. This allows access to pointers of the AST created by `apollo-parser`.

  [lrlna]: https://github.com/lrlna
  [pull/251]: https://github.com/apollographql/apollo-rs/pull/251

# [0.2.8](https://crates.io/crates/apollo-parser/0.2.8) - 2022-06-10

## Fixes
- **Use recursion limit both for selection set and field parsing - [garypen] and [lrlna], [pull/244]**

  This properly unifies the limits around recursion for both:
    - selection sets
    - fields

  The tests are expanded and properly exercise the various possible outcomes
  with recursion limits.Fixes a bug with

  [garypen]: https://github.com/garypen
  [lrlna]: https://github.com/lrlna
  [pull/244]: https://github.com/apollographql/apollo-rs/pull/244

# [0.2.7](https://crates.io/crates/apollo-parser/0.2.7) - 2022-06-08

## Features
- **Resource bound parsing execution - [garypen], [pull/239] closes [issue/225]**

  Introduce recursion limit enforced during SelectionSet parsing.

  There is now a default limit (4_096) applied to parsers during SelectionSet
  parsing to help prevent stack overflows. This limit can be set manually when
  creating a parser by using the new fn, `Parser::with_recursion_limit()`.
  Details about recursion consumption can be retrieved using the new fn
  `SyntaxTree::recursion_limit()`.  Recursion limit details are also output as
  part of the AST debug output when printing a `SyntaxTree`.

  [garypen]: https://github.com/garypen
  [pull/239]: https://github.com/apollographql/apollo-rs/pull/239
  [issue/225]: https://github.com/apollographql/apollo-rs/issues/225

# [0.2.6](https://crates.io/crates/apollo-parser/0.2.6) - 2022-05-24

## Fixes
- **lex escaped characters in StringValue tokens - [bnjjj], [pull/228] closes [issue/227], [issue/229]**

  StringValues with correctly escaped quotation marks, e.g. `{ name(id: "\"escaped\"") }`
  would error and not lex correctly. Additionally, invalid escapes in string
  values, e.g. `{ name(id: "escaped \a") }` should have an error created in the
  lexer. Both issues are fixed, and correctly bubble up to the parser.


  [bnjjj]: https://github.com/bnjjj
  [pull/228]: https://github.com/apollographql/apollo-rs/pull/228
  [issue/227]: https://github.com/apollographql/apollo-rs/issues/227
  [issue/229]: https://github.com/apollographql/apollo-rs/issues/229

# [0.2.5](https://crates.io/crates/apollo-parser/0.2.5) - 2022-04-01

> Important: 1 breaking change below, indicated by **BREAKING**

## BREAKING
- **GraphQL Int Values are cast to i32 - [bnjjj], [pull/197]**
  AST's Int Values have an `Into` implementation to their Rust type. They were
  previously converted to i64, which is not compliant with the spec. Int Values
  are now converted to i32.
  ```rust
  if let ast::Value::IntValue(val) =
      argument.value().expect("Cannot get argument value.")
  {
      let i: i32 = val.into();
  }
  ```
  [bnjjj]: https://github.com/bnjjj
  [pull/197]: https://github.com/apollographql/apollo-rs/pull/197

## Features
- **Adds a .text() method to ast::DirectiveLocation - [bnjjj], [pull/197]**
  `DirectiveLocation` can now additionally be accessed with a `.text()` method.

  ```rust
  let schema = r#"directive @example on FIELD | FRAGMENT_SPREAD | INLINE_FRAGMENT"#;
  let parser = Parser::new(schema);
  let ast = parser.parse();

  assert!(ast.errors.is_empty());

  let document = ast.document();
  for definition in document.definitions() {
      if let ast::Definition::DirectiveDefinition(dir_def) = definition {
          let dir_locations: Vec<String> = dir_def
              .directive_locations()
              .unwrap()
              .directive_locations()
              .map(|loc| loc.text().unwrap().to_string())
              .collect();
          assert_eq!(
              dir_locations,
              ["FIELD", "FRAGMENT_SPREAD", "INLINE_FRAGMENT"]
          );
          return;
      }
  }
  ```

  [bnjjj]: https://github.com/bnjjj
  [pull/197]: https://github.com/apollographql/apollo-rs/pull/197
# [0.2.4](https://crates.io/crates/apollo-parser/0.2.4) - 2022-03-07
## Fixes
- **correctly parse Arguments Definition - [bnjjj], [pull/187] closes [issue/186]**

  `apollo-parser` was creating ARGUMENTS instead of ARGUMENTS_DEFINITION nodes
  when parsing Arguments Definitions. This change fixes the incorrect parsing
  and allows to iterate over arguments definitions returned by the AST.

  [bnjjj]: https://github.com/bnjjj
  [pull/187]: https://github.com/apollographql/apollo-rs/pull/187
  [issue/186]: https://github.com/apollographql/apollo-rs/issues/186

- **Add STRING_VALUE node to DESCRIPTION - [bnjjj], [pull/188] closes [issue/185]**

  DESCRIPTION nodes are composed of STRING_VALUE nodes. The description string
  was previously simply added to the DESCRIPTION node which was not spec
  compliant.

  [bnjjj]: https://github.com/bnjjj
  [pull/188]: https://github.com/apollographql/apollo-rs/pull/188
  [issue/185]: https://github.com/apollographql/apollo-rs/issues/185

- **Schema Definition has a description - [bnjjj], [pull/188] closes [issue/185]**

  `apollo-parser` was parsing descriptions in Schema Definitions, but the
  graphql ungrammar did not account for a description node. This updates the
  ungrammar, and provides an accessor method to Schema Definition's description.

  [bnjjj]: https://github.com/bnjjj
  [pull/188]: https://github.com/apollographql/apollo-rs/pull/188
  [issue/185]: https://github.com/apollographql/apollo-rs/issues/185

- **Add `repeatable` keyword to GraphQL ungrammar - [bnjjj], [pull/189]**

  `repeatable` keyword was not able to be accessed programmatically from the
  parsed AST for Directive Definitions, this is now fixed.

  [bnjjj]: https://github.com/bnjjj
  [pull/189]: https://github.com/apollographql/apollo-rs/pull/189

# [0.2.3](https://crates.io/crates/apollo-parser/0.2.3) - 2022-02-17
## Features
- **expose Lexer as a pub struct - [bnjjj], [pull/168]**

  The `Lexer` in `apollo-parser` is now a publicly available interface.

  ```rust
  use apollo_parser::Lexer;

  let query = "
  {
      animal
      ...snackSelection
      ... on Pet {
        playmates {
          count
        }
      }
  }
  ";
  let lexer = Lexer::new(query);
  assert_eq!(lexer.errors().len(), 0);

  let tokens = lexer.tokens();
  ```

  [bnjjj]: https://github.com/bnjjj
  [pull/168]: https://github.com/apollographql/apollo-rs/pull/168

## Fixes
- **add a getter for Directives in Variable Definitions - [lrlna], [pull/172]**

  While the parser was correctly parsing and accounting for directives in a
  variable definition, the getter for Directives in VariableDefinition type in the
  AST was missing. This commit makes an addition to the graphql ungrammar, and by
  extension the generated AST nodes API.

  [lrlna]: https://github.com/lrlna
  [pull/172]: https://github.com/apollographql/apollo-rs/pull/172

# [0.2.2](https://crates.io/crates/apollo-parser/0.2.2) - 2022-02-11
## Fixes
- **create an error when description preceeds operation definition and proceed parsing - [MidasLamb], [pull/158]/ [lrlna], [pull/160]**

  According to the spec Operation Definitions don't currently allow for
  descriptions.

  ```graphql
  "this description is not allowed"
  {
    name
    age
  }
  ```

  When a description was added before an operation, the parser
  would continuously try to register the error without removing it from the list
  of valid tokens. This fix removes the incorrect token, and continuous parsing
  an OperationDefinition.

  [MidasLamb]: https://github.com/MidasLamb
  [lrlna]: https://github.com/lrlna
  [pull/158]: https://github.com/apollographql/apollo-rs/pull/158
  [pull/160]: https://github.com/apollographql/apollo-rs/pull/160

- **Correctly parse an Inline Fragment when type condition is absent - [bnjjj], [pull/164]**

  The following inline fragment would previously be incorrectly parsed as a FragmentSpread when in reality it's an Inline Fragment:
  ```graphql
  query HeroForEpisode {
    ... @tag(name: "team-customers") { # an inline fragment
      primaryFunction
    }
  }
  ```

  This has now been fixed.

  [bnjjj]: https://github.com/bnjjj
  [pull/164]: https://github.com/apollographql/apollo-rs/pull/164
# [0.2.1](https://crates.io/crates/apollo-parser/0.2.1) - 2022-01-26
## Fixes
- **fix(apollo-parser): add ignored tokens to TYPE nodes in correct place - [lrlna], [issue/143] [pull/153]**

  This fixes the location of ignored tokens (COMMA, WHITESPACE) inside a TYPE node.

  Before this commit this sort of query

  ```graphql
  mutation MyMutation($custId: Int!, $b: String) {
    myMutation(custId: $custId)
  }
  ```

  would result the `ast.document.to_string()` to have this output:

  ```graphql
  mutation MyMutation($custId: , Int!$b:  String) {
      myMutation(custId: $custId)
  }
  ```

  which is incorrect. The `to_string()` now results in the exact same output, as
  the AST created is correct.

  [lrlna]: https://github.com/lrlna
  [issue/143]: https://github.com/apollographql/apollo-rs/issues/143
  [pull/153]: https://github.com/apollographql/apollo-rs/pull/153

- **fix(apollo-parser): bump BANG token when creating NON_NULL_TYPE - [lrlna], [issue/142] [pull/146]**

  We are missing BANG token in the AST when a NON_NULL_TYPE gets created.
  Although the node created is indeed NON_NULL_TYPE, it's also important to keep
  the original set of tokens. The followin example now works:

  ```rust
  let mutation = r#"
  mutation MyMutation($custId: Int!) {
    myMutation(custId: $custId)
  }"#;

  let parser = Parser::new(mutation);
  let ast = parser.parse();
  assert_eq!(ast.errors.len(), 0);

  let doc = ast.document();
  assert_eq(&doc, &mutation);
  ```

  [lrlna]: https://github.com/lrlna
  [issue/142]: https://github.com/apollographql/apollo-rs/issues/142
  [pull/146]: https://github.com/apollographql/apollo-rs/pull/146

# [0.2.0](https://crates.io/crates/apollo-parser/0.2.0) - 2021-12-22
## Breaking
- **impl Iterator for ast.errors() - [o0Ignition0o], [issue/119] [pull/120]**

  `ast.errors()` now return an Iterator. This makes it a bit easier for users to process any errors returned by the Parser. Below is the new usage example:

  ```rust

  let query = r#"
  type Query {
      "A simple type for getting started!"
      hello: String
      cats(cat: [String]! = ): [String]!
  }"#;


  let parser = Parser::new(&query);
  let ast = parser.parse();

  assert!(ast.errors.len(), 1);

  for err in ast.errors() { // no longer need to .iter() on this
      // process errors in a way that's useful for your implementation
      dbg!(&err);
  }
  ```

  [o0Ignition0o]: https://github.com/o0Ignition0o
  [issue/119]: https://github.com/apollographql/apollo-rs/issues/119
  [pull/120]: https://github.com/apollographql/apollo-rs/pull/120

## Fixes
- **fix: properly create TYPE's NAMED_TYPE, LIST_TYPE, NON_NULL_TYPE - [lrlna], [issue/125] [pull/127]**

  Whenever a NAMED_TYPED, LIST_TYPE, NON_NULL_TYPE are created, they are
  automatically get created as part of the TYPE node, so we do not need to start
  it manually. This fix makes it possible to once again do:

  ```rust
  if let ast::Type::NamedType(name) = var.ty().unwrap() {
      assert_eq!(name.name().unwrap().text().as_ref(), "Int")
  }
  ```
  [lrlna]: https://github.com/lrlna
  [issue/125]: https://github.com/apollographql/apollo-rs/issues/125
  [pull/127]: https://github.com/apollographql/apollo-rs/pull/127

- **fix: create an error when SelectionSet is empty in operation definition - [lrlna], [pull/134]**

  An Operation Definition must have a selection set with values, so this query
  `query {}` should also come with an error.

  [lrlna]: https://github.com/lrlna
  [pull/134]: https://github.com/apollographql/apollo-rs/pull/134

- **fix: variable definition can have a LIST_TYPE - [lrlna], [issue/131] [pull/135]**

  Variable definition was previously not accepting a LIST_TYPE, which is
  incorrect. This commit fixes this issue.

  [lrlna]: https://github.com/lrlna
  [issue/131]: https://github.com/apollographql/apollo-rs/issues/131
  [pull/135]: https://github.com/apollographql/apollo-rs/pull/135

## Maintenance
- **chore: typo in README - [lrlna], [c598d3]**

  [lrlna]: https://github.com/lrlna
  [c598d3]: https://github.com/apollographql/apollo-rs/commit/c598d33bc9e80f767804bf5a88a7a1d6f400e832

- **fuzzing for apollo-parser - [Geal], [pull/122]**

  The fuzz test checks for lexer and parser errors and stops early.

  The following fuzz-encountered errors are fixed:
  - panics on the following input:
  ```
  "
  ```
  - crash on partial block string opening token
  ```
  ""
  ```
  - infinite loop on unfollowed 'extend' ident

  The parser fuzzer catches errors in the lexer and returns early. It
  will not avoid infinite loops and running out of memory in the lexer.

  [Geal]: https://github.com/Geal
  [pull/122]: https://github.com/apollographql/apollo-rs/pull/122

- **chore: run clippy in CI on benchmark directories - [lrlna], [pull/123]**

  [lrlna]: https://github.com/lrlna
  [pull/123]: https://github.com/apollographql/apollo-rs/pull/123

- **chore: add tests for untermiated strings and invalid type system extensions - [lrlna], [pull/124]**

  Follows up on [#122] and adds tests for the incorrectly lexed and parsed
  inputs that fuzzing discovered.

  This commit also changes logic around having an "unexpected end of data" for
  `""` string. This now gets lexed into a `StringValue` token.

  [lrlna]: https://github.com/lrlna
  [pull/124]: https://github.com/apollographql/apollo-rs/pull/124
  [#122]: https://github.com/apollographql/apollo-rs/pull/122

- **chore: allow dead code in xtask's ast_src - [lrlna], [pull/128]**

  [lrlna]: https://github.com/lrlna
  [pull/128]: https://github.com/apollographql/apollo-rs/pull/128

- **chore: add a test for nested SELECTION_SETs - [lrlna], [pull/137]**

  This will mostly act as an example in case users are looking for how to work
  with nested selections and get their FIELD/INLINE_FRAGMENT/FRAGMENT_SPREAD.

  [lrlna]: https://github.com/lrlna
  [pull/137]: https://github.com/apollographql/apollo-rs/pull/137
