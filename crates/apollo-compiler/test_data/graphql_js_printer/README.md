# graphql-js printer expectations

Test data for `to_graphql_js_string()`, the graphql-js compatible printer
(`tests/graphql_js_printer.rs`).

* `input/` contains hand-written documents exercising the formatting rules
  (argument, list, object and variable-definition wrapping, string escaping,
  block strings, fragments, every operation type, every type system definition).
* `expected/{ok,diagnostics,input}/` contains, for every `.graphql` file in
  `../ok`, `../diagnostics` and `input/`, the output of `print(parse(input))`
  from **graphql-js 17.0.2**. `expected/unsupported.txt` lists the inputs
  graphql-js cannot parse.
* `generate.js` regenerates `expected/` from a local copy of graphql-js:

  ```sh
  npm install graphql@17.0.2
  node generate.js ./node_modules/graphql
  ```

Before printing, the generator normalizes the `block` flag of every
`StringValue` node to what `apollo_compiler::ast` can represent: `false` for
values (the AST does not record block-string syntax, see
[apollo-rs#1120](https://github.com/apollographql/apollo-rs/issues/1120)) and
`isPrintableAsBlockString(value)` for descriptions (the heuristic graphql-js's
own `printSchema` uses). `node generate.js <graphql> --raw <out_dir>` skips the
normalization, which is useful to see how far the Rust output is from an
unmodified graphql-js.

Differences between graphql-js 16.14.2 and 17.0.2 that affect these
expectations (17 is implemented): object values print as `{ a: 1 }` instead of
`{a: 1}`, object and list values wrap onto one line per item when the inline
form exceeds 80 characters, and consequently variable definitions wrap when
one of them contains such a multi-line default value.

Do not regenerate these files from the Rust output: they are the reference,
not a snapshot.
