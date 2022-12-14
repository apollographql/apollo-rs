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
* Ergonomic API on top the AST created by `apollo-parser`
* GraphQL validation and diagnostics reporting
    * Validation is a work in progress, stay tuned for further validation rules implementation.

## Getting started
Add this to your `Cargo.toml` to start using `apollo-compiler`:
```toml
# Just an example, change to the necessary package version.
[dependencies]
apollo-compiler = "0.4.0"
```

Or using [cargo-edit]:
```bash
cargo add apollo-compiler
```

## Usage
`apollo-compiler` is built using [`salsa`] to provide a
memoised query system on top of the AST produced by `apollo-parser`.
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
compiler.create_document(input, "document.graphql");

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
use apollo_compiler::{ApolloCompiler, hir, DocumentDatabase};
use miette::Result;

fn main() -> Result<()> {
    let schema_input = r#"
    type Query {
      topProducts: Product
      customer: User
    }

    type Product {
      type: String
      price(setPrice: Int): Int
    }

    type User {
      id: ID
      name: String
      profilePic(size: Int): URL
    }

    scalar URL @specifiedBy(url: "https://tools.ietf.org/html/rfc3986")
    "#;
    let query_input = r#"
    query getProduct {
      topProducts {
          type
      }
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
    let _schema_id = compiler.create_schema(schema_input, "schema.graphql");
    let query_id = compiler.create_executable(query_input, "query.graphql");

    let diagnostics = compiler.validate();
    for diagnostic in &diagnostics {
        println!("{}", diagnostic);
    }
    assert!(diagnostics.is_empty());

    let op = compiler.db.find_operation_by_name(query_id, String::from("getProduct"))
        .expect("getProduct query does not exist");
    let fragment_in_op: Vec<hir::FragmentDefinition> = op.selection_set().selection().iter().filter_map(|sel| match sel {
        hir::Selection::FragmentSpread(frag) => {
            Some(frag.fragment(&compiler.db)?.as_ref().clone())
        }
        _ => None
    }).collect();

    let fragment_fields: Vec<hir::Field> = fragment_in_op.iter().flat_map(|frag| frag.selection_set().fields()).collect();
    let field_ty: Vec<String> = fragment_fields
        .iter()
        .filter_map(|f| Some(f.ty(&compiler.db)?.name()))
        .collect();
    assert_eq!(field_ty, ["ID", "String", "URL"]);
    Ok(())
}
```

#### Get a directive defined on a field used in a query operation definition.
```rust
use apollo_compiler::{ApolloCompiler, hir, HirDatabase};
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
    compiler.create_schema(schema_input, "schema.graphql");
    let query_id = compiler.create_executable(query_input, "query.graphql");

    let diagnostics = compiler.validate();
    for diagnostic in &diagnostics {
        println!("{}", diagnostic);
    }
    assert!(diagnostics.is_empty());

    let operations = compiler.db.operations(query_id);
    let get_product_op = operations
        .iter()
        .find(|op| op.name() == Some("getProduct"))
        .expect("getProduct query does not exist");
    let op_fields = get_product_op.fields(&compiler.db);

    let in_stock_field = op_fields
        .iter()
        .find(|f| f.name() == "topProducts")
        .expect("topProducts field does not exist")
        .selection_set()
        .field("inStock")
        .expect("inStock field does not exist")
        .field_definition(&compiler.db)
        .expect("field definition does not exist");
    let in_stock_directive: Vec<&str> = in_stock_field
        .directives()
        .iter()
        .map(|dir| dir.name())
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
compiler.create_document(input, "document.graphql");

let diagnostics = compiler.validate();
for diagnostic in &diagnostics {
    println!("{}", diagnostic)
}
assert_eq!(diagnostics.len(), 5)
```

## License
Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or https://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or https://opensource.org/licenses/MIT)

at your option.

[cargo-edit]: https://github.com/killercup/cargo-edit
[`salsa`]: https://docs.rs/salsa/latest/salsa/
