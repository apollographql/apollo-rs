<div align="center">
  <h1><code>apollo-compiler</code></h1>

  <p>
    <strong>A query-based compiler for the GraphQL query language.</strong>
  </p>
  <p>
    <a href="https://crates.io/crates/apollo-compiler">
        <img src="https://img.shields.io/crates/v/apollo-compiler.svg?style=flat-square" alt="Crates.io version" />
    </a>
    <a href="https://crates.io/crates/apollo-compiler">
        <img src="https://img.shields.io/crates/d/apollo-compiler.svg?style=flat-square" alt="Download" />
    </a>
    <a href="https://docs.rs/apollo-compiler/">
        <img src="https://img.shields.io/static/v1?label=docs&message=apollo-compiler&color=blue&style=flat-square" alt="docs.rs docs" />
    </a>
  </p>
</div>

## Features

* A (comparatively) low-level AST for GraphQL grammar,
  and high-level representation of `Schema` and `ExecutableDocument`.
* All three can be parsed (using `apollo-parser` internally),
  created or modified programatically,
  and serialized.
* Validation of schemas and executable documents, as defined [in the GraphQL specification][val].
* Execution of the [schema introspection][introsp] portion of queries.

[val]: https://spec.graphql.org/October2021/#sec-Validation
[introsp]: https://spec.graphql.org/October2021/#sec-Introspection

## Getting started
Add the dependency to start using `apollo-compiler`:
```bash
cargo add apollo-compiler
```

Or add this to your `Cargo.toml` for a manual installation:

```toml
# Just an example, change to the necessary package version.
[dependencies]
apollo-compiler = "1.28.0"
```

## Rust versions

`apollo-compiler` is tested on the latest stable version of Rust.
Older version may or may not be compatible.

## Usage

You can get started with `apollo-compiler`:
```rust
use apollo_compiler::Schema;
use apollo_compiler::ExecutableDocument;

let sdl = r#"
  type Query {
    field: Int
  }
"#;
let query = "{ field }";

/// In case of validation errors, the panic message will be nicely formatted
/// to point at relevant parts of the source file(s)
let schema = Schema::parse_and_validate(sdl, "sdl.graphql").unwrap();
let doc = ExecutableDocument::parse_and_validate(&schema, query, "query.graphql").unwrap();
```

### Examples
#### Accessing fragment definition field types

```rust
use apollo_compiler::{Schema, ExecutableDocument, Node, executable};

let schema_input = r#"
type User {
  id: ID
  name: String
  profilePic(size: Int): URL
}

schema { query: User }

scalar URL @specifiedBy(url: "https://tools.ietf.org/html/rfc3986")
"#;
let query_input = r#"
query getUser {
  ... vipCustomer
}

#fragment definition where we want to know the field types.
fragment vipCustomer on User {
  id
  name
  profilePic(size: 50)
}
"#;

let schema = Schema::parse_and_validate(schema_input, "schema.graphql").unwrap();
let document = ExecutableDocument::parse_and_validate(&schema, query_input, "query.graphql")
    .unwrap();

let op = document.operations.get(Some("getUser")).expect("getUser query does not exist");
let fragment_in_op = op.selection_set.selections.iter().filter_map(|sel| match sel {
    executable::Selection::FragmentSpread(spread) => {
        Some(document.fragments.get(&spread.fragment_name)?.as_ref())
    }
    _ => None
}).collect::<Vec<&executable::Fragment>>();

let fragment_fields = fragment_in_op.iter().flat_map(|frag| {
    frag.selection_set.fields()
}).collect::<Vec<&Node<executable::Field>>>();
let field_ty = fragment_fields
    .iter()
    .map(|f| f.ty().inner_named_type().as_str())
    .collect::<Vec<&str>>();
assert_eq!(field_ty, ["ID", "String", "URL"]);
```

#### Get a directive defined on a field used in a query operation definition.
```rust
use apollo_compiler::{Schema, ExecutableDocument, Node, executable};

let schema_input = r#"
type Query {
  topProducts: Product
}

type Product {
  inStock: Boolean @join__field(graph: INVENTORY)
  name: String @join__field(graph: PRODUCTS)
}

enum join__Graph {
  INVENTORY,
  PRODUCTS,
}
directive @join__field(graph: join__Graph) on FIELD_DEFINITION
"#;
let query_input = r#"
query getProduct {
  topProducts {
    inStock
  }
}
"#;

let schema = Schema::parse_and_validate(schema_input, "schema.graphql").unwrap();
let document = ExecutableDocument::parse_and_validate(&schema, query_input, "query.graphql")
    .unwrap();

let get_product_op = document
    .operations
    .get(Some("getProduct"))
    .expect("getProduct query does not exist");

let in_stock_field = &get_product_op
    .selection_set
    .fields()
    .find(|f| f.name == "topProducts")
    .expect("topProducts field does not exist")
    .selection_set
    .fields()
    .find(|f| f.name == "inStock")
    .expect("inStock field does not exist")
    .definition;
let in_stock_directive: Vec<_> = in_stock_field
    .directives
    .iter()
    .map(|dir| &dir.name)
    .collect();
assert_eq!(in_stock_directive, ["join__field"]);
```

#### Printing diagnostics for a faulty GraphQL document
```rust
use apollo_compiler::parser::Parser;

let input = "{ ... }";

if let Err(diagnostics) = Parser::new().parse_mixed_validate(input, "document.graphql") {
    println!("{diagnostics}")
}
```

## License
Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <https://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <https://opensource.org/licenses/MIT>)

at your option.

[`salsa`]: https://docs.rs/salsa/latest/salsa/
