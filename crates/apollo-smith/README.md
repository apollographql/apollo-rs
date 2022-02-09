# apollo-smith

 <div align="center">
   <h1><code>apollo-smith</code></h1>

   <p>
     <strong>A GraphQL test case generator.</strong>
   </p>
   <p>
     <a href="https://crates.io/crates/apollo-smith">
         <img src="https://img.shields.io/crates/v/apollo-smith.svg?style=flat-square" alt="Crates.io" />
     </a>
     <a href="https://crates.io/crates/apollo-smith">
         <img src="https://img.shields.io/crates/d/apollo-smith.svg?style=flat-square" alt="Download" />
     </a>
     <a href="https://docs.rs/apollo-smith/">
         <img src="https://img.shields.io/static/v1?label=docs&message=apollo-smith&color=blue&style=flat-square" alt="docs.rs docs" />
     </a>
   </p>
 </div>

## Usage

First, use [`cargo fuzz`](https://github.com/rust-fuzz/cargo-fuzz) to define
a new fuzz target:

```shell
$ cargo fuzz add my_apollo_smith_fuzz_target
```

Next, add `apollo-smith` to your dependencies:

```toml
## fuzz/Cargo.toml

[dependencies]
apollo-smith = "0.1.0"
```

Then, define your fuzz target so that it takes arbitrary
`&[u8]`s as an argument, and create a `Document` thanks to the `DocumentBuilder`.
The method `DocumentBuilder::new` takes a `&mut Unstructured` you can create thanks to
the input bytes (`&[u8]`) provided in arguments and the [arbitrary crate](https://docs.rs/arbitrary).
You then can call `finish()` method on your `DocumentBuilder` instance to get the GraphQL document as a `String`.

```rust,compile_fail
// fuzz/fuzz_targets/my_apollo_smith_fuzz_target.rs

#![no_main]

use libfuzzer_sys::fuzz_target;
use arbitrary::Unstructured;
use apollo_smith::DocumentBuilder;

fuzz_target!(|input: &[u8]| {
    let mut u = Unstructured::new(input);
    let gql_doc = DocumentBuilder::new(&mut u)?;
    let document = gql_doc.finish();
    let document_str = String::from(document);

    // Your code here...
});
```

Finally, start fuzzing:

```shell
$ cargo +nightly fuzz run my_apollo_smith_fuzz_target
```

You can also use existing target:

```shell
$ cargo +nightly fuzz run parser
```

## Credits

The design and implementation strategy of apollo-smith has been inspired by
[this awesome article](https://fitzgeraldnick.com/2020/08/24/writing-a-test-case-generator.html).

## Limitations

- Syntactly correct, but not semantically correct (example when creating a default value argument you can have something like `value: Int = "test"`)
  - Usage of unknown directives
  - Default values for fields are not semantically correct
  - Input values too
  - We could have duplicated names in variables
  - Add more special characters for description (", ', )
- Recursive object type not already supported (example : `myType { inner: myType }`)
