<div align="center">
  <h1><code>apollo-rs</code></h1>

  <p>
    <strong>Rust tooling for low-level manipulation of the GraphQL language.</strong>
  </p>
</div>

# Tools included

This project is intended to house a number of tools related to the low-level
workings of GraphQL according to the [GraphQL
specification (June 2018)](https://spec.graphql.org/draft). Nothing in
these libraries is specific to Apollo, and can freely be used by other
projects which need standards-compliant GraphQL tooling written in Rust. The
following crates currently exist:

* [**`apollo-encoder`**](crates/apollo-encoder) - a library to generate GraphQL code.
* [**`apollo-parser`**](crates/apollo-parser) - a library to parse the GraphQL
  query language.

# Parser

## Example
```rust
use apollo_parser::Parser;
use apollo_parser::ast::{Definition, ObjectTypeDefinition};

let input = "
type ProductDimension {
  size: String
  weight: Float @tag(name: \"hi from inventory value type field\")
}
";
let parser = Parser::new(input);
let ast = parser.parse();
assert!(ast.errors().is_empty());

let doc = ast.document();

for def in doc.definitions() {
    if let Definition::ObjectTypeDefinition(object_type) = def {
        assert_eq!(object_type.name().unwrap().text(), "ProductDimension");
        for field_def in object_type.fields_definition().unwrap().field_definitions() {
            println!("{}", field_def.name().unwrap().text()); // size weight
        }
    }
}
```

# License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or https://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or https://opensource.org/licenses/MIT)

at your option.