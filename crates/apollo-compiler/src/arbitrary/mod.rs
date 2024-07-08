//! Supporting library for generating GraphQL documents
//! in a [fuzzing target](https://rust-fuzz.github.io/book/introduction.html)

mod arbitrary_executable;
mod common;
mod entropy;

pub use self::arbitrary_executable::arbitrary_valid_executable_document;
// TODO: should this be public? Maybe for subgraph generation in apollo-federation crate?
// pub use self::entropy::Entropy;
// pub use self::entropy::Int;
