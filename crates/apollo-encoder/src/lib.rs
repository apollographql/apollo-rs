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
//! use apollo_encoder::{Schema, Field, UnionDef, EnumValue, Directive, EnumDef, Type_};
//! use indoc::indoc;
//!
//! let mut schema = Schema::new();
//!
//! // Create a Directive Definition.
//! let mut directive = Directive::new("provideTreat".to_string());
//! directive.description(Some("Ensures cats get treats.".to_string()));
//! directive.location("OBJECT".to_string());
//! directive.location("FIELD_DEFINITION".to_string());
//! directive.location("INPUT_FIELD_DEFINITION".to_string());
//! schema.directive(directive);

//! // Create an Enum Definition
//! let mut enum_ty_1 = EnumValue::new("CatTree".to_string());
//! enum_ty_1.description(Some("Top bunk of a cat tree.".to_string()));
//! let enum_ty_2 = EnumValue::new("Bed".to_string());
//! let mut enum_ty_3 = EnumValue::new("CardboardBox".to_string());
//! enum_ty_3.deprecated(Some("Box was recycled.".to_string()));
//!
//! let mut enum_def = EnumDef::new("NapSpots".to_string());
//! enum_def.description(Some("Favourite cat\nnap spots.".to_string()));
//! enum_def.value(enum_ty_1);
//! enum_def.value(enum_ty_2);
//! enum_def.value(enum_ty_3);
//! schema.enum_(enum_def);
//! // Union Definition
//! let mut union_def = UnionDef::new("Cat".to_string());
//! union_def.description(Some(
//!     "A union of all cats represented within a household.".to_string(),
//! ));
//! union_def.member("NORI".to_string());
//! union_def.member("CHASHU".to_string());
//! schema.union(union_def);
//!
//! assert_eq!(
//!     schema.finish(),
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

pub use directive_def::Directive;
pub use enum_def::EnumDef;
pub use enum_value::EnumValue;
pub use field::Field;
pub use field_value::Type_;
pub use input_field::InputField;
pub use input_object_def::InputObjectDef;
pub use input_value::InputValue;
pub use interface_def::InterfaceDef;
pub use object_def::ObjectDef;
pub use scalar_def::ScalarDef;
pub use schema::Schema;
pub use schema_def::SchemaDef;
pub use string_value::StringValue;
pub use union_def::UnionDef;
