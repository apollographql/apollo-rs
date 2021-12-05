//! <div align="center">
//!   <h1><code>apollo-encoder</code></h1>
//!
//!   <p>
//!     <strong>A library to generate GraphQL Code, SDL.</strong>
//!   </p>
//!   <p>
//!     <a href="https://crates.io/crates/apollo-encoder">
//!         <img src="https://img.shields.io/crates/v/apollo-encoder.svg?style=flat-square" alt="Crates.io" />
//!     </a>
//!     <a href="https://crates.io/crates/apollo-encoder">
//!         <img src="https://img.shields.io/crates/d/apollo-encoder.svg?style=flat-square" alt="Download" />
//!     </a>
//!     <a href="https://docs.rs/apollo-encoder/">
//!         <img src="https://img.shields.io/static/v1?label=docs&message=apollo-encoder&color=blue&style=flat-square" alt="docs.rs docs" />
//!     </a>
//!   </p>
//! </div>
//!
//! For more information on GraphQL Schema Types, please refer to [official
//! documentation](https://graphql.org/learn/schema/).
//!
//! ## Getting started
//! Add this to your `Cargo.toml` to start using `apollo-encoder`:
//! ```toml
//! # Just an example, change to the necessary package version.
//! [dependencies]
//! apollo-encoder = "0.1.0"
//! ```
//!
//! Or using [cargo-edit]:
//! ```bash
//! cargo add apollo-encoder
//! ```
//!
//! ## Example
//! ```rust
//! use apollo_encoder::{Schema, FieldBuilder, UnionDefBuilder, EnumValueBuilder, DirectiveBuilder, EnumDefBuilder, Type_};
//! use indoc::indoc;
//!
//! let directive = DirectiveBuilder::new("provideTreat")
//!     .description("Ensures cats get treats.")
//!     .location("OBJECT")
//!     .location("FIELD_DEFINITION")
//!     .location("INPUT_FIELD_DEFINITION")
//!     .build();
//!
//! let enum_def = {
//!     let enum_ty_1 = EnumValueBuilder::new("CatTree")
//!         .description("Top bunk of a cat tree.")
//!         .build();
//!     let enum_ty_2 = EnumValueBuilder::new("Bed").build();
//!     let enum_ty_3 = EnumValueBuilder::new("CardboardBox")
//!         .deprecated("Box was recycled.")
//!         .build();
//!
//!     EnumDefBuilder::new("NapSpots")
//!         .description("Favourite cat\nnap spots.")
//!         .value(enum_ty_1)
//!         .value(enum_ty_2)
//!         .value(enum_ty_3)
//!         .build()
//! };
//!
//! let union_def = UnionDefBuilder::new("Cat")
//!     .description("A union of all cats represented within a household.")
//!     .member("NORI")
//!     .member("CHASHU")
//!     .build();
//!
//! let schema = Schema::new()
//!     .directive(directive)
//!     .enum_(enum_def)
//!     .union(union_def)
//!     .finish();
//!
//! assert_eq!(
//!     schema,
//!     indoc! { r#"
//!         "Ensures cats get treats."
//!         directive @provideTreat on OBJECT | FIELD_DEFINITION | INPUT_FIELD_DEFINITION
//!         """
//!         Favourite cat
//!         nap spots.
//!         """
//!         enum NapSpots {
//!           "Top bunk of a cat tree."
//!           CatTree
//!           Bed
//!           CardboardBox @deprecated(reason: "Box was recycled.")
//!         }
//!         "A union of all cats represented within a household."
//!         union Cat = NORI | CHASHU
//!     "# }
//! );
//! ```
//!
//! ## License
//! Licensed under either of
//!
//! - Apache License, Version 2.0 ([LICENSE-APACHE] or <https://www.apache.org/licenses/LICENSE-2.0>)
//! - MIT license ([LICENSE-MIT] or <https://opensource.org/licenses/MIT>)
//!
//! at your option.
//!
//! [cargo-edit]: https://github.com/killercup/cargo-edit
//! [LICENSE-APACHE]: https://github.com/apollographql/apollo-rs/blob/main/crates/apollo-parser/LICENSE-APACHE
//! [LICENSE-MIT]: https://github.com/apollographql/apollo-rs/blob/main/crates/apollo-parser/LICENSE-MIT

#![forbid(unsafe_code)]
#![deny(missing_debug_implementations, nonstandard_style)]
#![warn(missing_docs, future_incompatible, unreachable_pub, rust_2018_idioms)]

mod directive_def;
mod enum_def;
mod enum_value;
mod field;
mod field_value;
mod input_field;
mod input_object_def;
mod input_value;
mod interface_def;
mod object_def;
mod scalar_def;
mod schema;
mod schema_def;
mod string_value;
mod union_def;

pub use directive_def::{Directive, DirectiveBuilder};
pub use enum_def::{EnumDef, EnumDefBuilder};
pub use enum_value::{EnumValue, EnumValueBuilder};
pub use field::{Field, FieldBuilder};
pub use field_value::Type_;
pub use input_field::{InputField, InputFieldBuilder};
pub use input_object_def::{InputObjectDef, InputObjectDefBuilder};
pub use input_value::{InputValue, InputValueBuilder};
pub use interface_def::{InterfaceDef, InterfaceDefBuilder};
pub use object_def::{ObjectDef, ObjectDefBuilder};
pub use scalar_def::{ScalarDef, ScalarDefBuilder};
pub use schema::Schema;
pub use schema_def::{SchemaDef, SchemaDefBuilder};
pub use string_value::StringValue;
pub use union_def::{UnionDef, UnionDefBuilder};
