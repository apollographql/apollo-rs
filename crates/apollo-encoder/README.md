<div align="center">
  <h1><code>apollo-encoder</code></h1>

  <p>
    <strong>A library to generate GraphQL Code, SDL.</strong>
  </p>
  <p>
    <a href="https://crates.io/crates/apollo-encoder">
        <img src="https://img.shields.io/crates/v/apollo-encoder.svg?style=flat-square" alt="Crates.io" />
    </a>
    <a href="https://crates.io/crates/apollo-encoder">
        <img src="https://img.shields.io/crates/d/apollo-encoder.svg?style=flat-square" alt="Download" />
    </a>
    <a href="https://docs.rs/apollo-encoder/">
        <img src="https://img.shields.io/static/v1?label=docs&message=apollo-encoder&color=blue&style=flat-square" alt="docs.rs docs" />
    </a>
  </p>
</div>

## Getting started

Add this to your `Cargo.toml` to start using `apollo-encoder`:

```toml
# Just an example, change to the necessary package version.
[dependencies]
apollo-encoder = "0.1.0"
```

Or using [cargo-edit]:

```bash
cargo add apollo-encoder
```

## Example

```rust
use apollo_encoder::{
    Argument, Directive, Document, Field, OperationDefinition, OperationType, Selection, SelectionSet, Type_, Value,
    VariableDefinition,
};
use indoc::indoc;
let mut document = Document::new();
let selection_set = {
    let sels = vec![
        Selection::Field(Field::new(String::from("first"))),
        Selection::Field(Field::new(String::from("second"))),
    ];
    let mut sel_set = SelectionSet::new();
    sels.into_iter().for_each(|sel| sel_set.selection(sel));
    sel_set
};
let var_def = VariableDefinition::new(
    String::from("variable_def"),
    Type_::List {
        ty: Box::new(Type_::NamedType {
            name: String::from("Int"),
        }),
    },
);
let mut new_op = OperationDefinition::new(OperationType::Query, selection_set);
let mut directive = Directive::new(String::from("testDirective"));
directive.arg(Argument::new(
    String::from("first"),
    Value::String("one".to_string()),
));
new_op.variable_definition(var_def);
new_op.directive(directive);
document.operation(new_op);
assert_eq!(
    document.to_string(),
    indoc! { r#"
        query($variable_def: [Int]) @testDirective(first: "one") {
          first
          second
        }
    "#}
);
```

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or https://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or https://opensource.org/licenses/MIT)

at your option.

[cargo-edit]: https://github.com/killercup/cargo-edit
