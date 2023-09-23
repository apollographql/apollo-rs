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
* Ergonomic API on top the CST created by `apollo-parser`
* GraphQL validation and diagnostics reporting
    * Validation is a work in progress, stay tuned for further validation rules implementation.

## Getting started
Add the dependency to start using `apollo-compiler`:
```bash
cargo add apollo-compiler
```

Or add this to your `Cargo.toml` for a manual installation:

```toml
# Just an example, change to the necessary package version.
[dependencies]
apollo-compiler = "0.11.0"
```

## Rust versions

`apollo-compiler` is tested on the latest stable version of Rust.
Older version may or may not be compatible.

## Usage
`apollo-compiler` is built using [`salsa`] to provide a
memoised query system on top of the CST produced by `apollo-parser`.
The idea is that all relationships between GraphQL types are pre-established and pre-computed, so you are able to always find the reference to say a field Type, or a Directive.

You can get started with `apollo-compiler`:
```rust
use apollo_compiler::ApolloCompiler;

let input = r#"
  interface Pet {
    name: String
  }

  type Dog implements Pet {
    name: String
    nickname: String
    barkVolume: Int
  }

  type Cat implements Pet {
    name: String
    nickname: String
    meowVolume: Int
  }

  union CatOrDog = Cat | Dog

  type Human {
    name: String
    pets: [Pet]
  }

  type Query {
    human: Human
  }
"#;

let mut compiler = ApolloCompiler::new();
compiler.add_document(input, "document.graphql");

let diagnostics = compiler.validate();
for diagnostic in &diagnostics {
    // this will pretty-print diagnostics using the miette crate.
    println!("{}", diagnostic);
}
assert!(diagnostics.is_empty());
```

### Examples
#### Accessing fragment definition field types

```rust
use apollo_compiler::{ApolloCompiler, ReprDatabase, Node, executable};
use miette::Result;

fn main() -> Result<()> {
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

    let mut compiler = ApolloCompiler::new();
    let _schema_id = compiler.add_type_system(schema_input, "schema.graphql");
    let query_id = compiler.add_executable(query_input, "query.graphql");

    let diagnostics = compiler.validate();
    for diagnostic in &diagnostics {
        println!("{}", diagnostic);
    }
    assert!(diagnostics.is_empty());

    let document = compiler.db.executable_document(query_id);
    let op = document.get_operation(Some("getUser")).expect("getUser query does not exist");
    let fragment_in_op: Vec<&executable::Fragment> = op.selection_set.selections.iter().filter_map(|sel| match sel {
        executable::Selection::FragmentSpread(spread) => {
            Some(document.fragments.get(&spread.fragment_name)?.as_ref())
        }
        _ => None
    }).collect();

    let fragment_fields: Vec<&Node<executable::Field>> = fragment_in_op.iter().flat_map(|frag| {
        frag.selection_set.fields()
    }).collect();
    let field_ty: Vec<&str> = fragment_fields
        .iter()
        .map(|f| f.ty().inner_named_type().as_str())
        .collect();
    assert_eq!(field_ty, ["ID", "String", "URL"]);
    Ok(())
}
```

#### Get a directive defined on a field used in a query operation definition.
```rust
use apollo_compiler::{ApolloCompiler, ReprDatabase, Node, executable};
use anyhow::{anyhow, Result};

fn main() -> Result<()> {
    let schema_input = r#"
    type Query {
      topProducts: Product
      name: String
      size: Int
    }

    type Product {
      inStock: Boolean @join__field(graph: INVENTORY)
      name: String @join__field(graph: PRODUCTS)
      price: Int
      shippingEstimate: Int
      upc: String!
      weight: Int
    }

    enum join__Graph {
      INVENTORY,
      PRODUCTS,
    }
    scalar join__FieldSet
    directive @join__field(graph: join__Graph, requires: join__FieldSet, provides: join__FieldSet) on FIELD_DEFINITION
    "#;
    let query_input = r#"
    query getProduct {
      size
      topProducts {
        name
        inStock
      }
    }
    "#;


    let mut compiler = ApolloCompiler::new();
    compiler.add_type_system(schema_input, "schema.graphql");
    let query_id = compiler.add_executable(query_input, "query.graphql");

    let diagnostics = compiler.validate();
    for diagnostic in &diagnostics {
        println!("{}", diagnostic);
    }
    let error_diagnostics = diagnostics
        .iter()
        .filter(|diag| diag.data.is_error())
        .collect::<Vec<_>>();
    assert!(error_diagnostics.is_empty());

    let document = compiler.db.executable_document(query_id);
    let get_product_op = document
        .get_operation(Some("getProduct"))
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
    Ok(())
}
```

#### Printing diagnostics for a faulty GraphQL document
```rust
use apollo_compiler::ApolloCompiler;

let input = r#"
query {
  cat {
    name
  }
}

query getPet {
  cat {
    owner {
      name
    }
  }
}

query getPet {
  cat {
    treat
  }
}

subscription sub {
  newMessage {
    body
    sender
  }
  disallowedSecondRootField
}

type Query {
  cat: Pet
}

type Subscription {
  newMessage: Result
}

interface Pet {
  name: String
}

type Dog implements Pet {
  name: String
  nickname: String
  barkVolume: Int
}

type Cat implements Pet {
  name: String
  nickname: String
  meowVolume: Int
}

union CatOrDog = Cat | Dog
"#;

let mut compiler = ApolloCompiler::new();
compiler.add_document(input, "document.graphql");

let diagnostics = compiler.validate();
for diagnostic in &diagnostics {
    println!("{}", diagnostic)
}
assert_eq!(diagnostics.len(), 9)
```

## License
Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or https://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or https://opensource.org/licenses/MIT)

at your option.

[`salsa`]: https://docs.rs/salsa/latest/salsa/
