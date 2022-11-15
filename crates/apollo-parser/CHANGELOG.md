# Changelog

All notable changes to `apollo-parser` will be documented in this file.

This project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

<!-- # [x.x.x] (unreleased) - 2021-mm-dd

> Important: X breaking changes below, indicated by **BREAKING**

## BREAKING

## Features

## Fixes

## Maintenance

## Documentation -->
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