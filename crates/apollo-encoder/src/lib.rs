#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]
#![deny(missing_debug_implementations, nonstandard_style)]
#![warn(missing_docs, future_incompatible, unreachable_pub, rust_2018_idioms)]

mod argument;
mod directive;
mod directive_def;
mod document;
mod enum_def;
mod enum_value;
mod field;
mod fragment;
#[cfg(feature = "apollo-parser")]
mod from_parser;
mod input_field;
mod input_object_def;
mod input_value;
mod interface_def;
mod object_def;
mod operation;
mod scalar_def;
mod schema_def;
mod selection_set;
mod string_value;
mod ty;
mod union_def;
mod value;
mod variable;

pub use argument::Argument;
pub use argument::ArgumentsDefinition;
pub use directive::Directive;
pub use directive_def::DirectiveDefinition;
pub use document::Document;
pub use enum_def::EnumDefinition;
pub use enum_value::EnumValue;
pub use field::{Field, FieldDefinition};
pub use fragment::{FragmentDefinition, FragmentSpread, InlineFragment, TypeCondition};
pub use input_field::InputField;
pub use input_object_def::InputObjectDefinition;
pub use input_value::InputValueDefinition;
pub use interface_def::InterfaceDefinition;
pub use object_def::ObjectDefinition;
pub use operation::{OperationDefinition, OperationType};
pub use scalar_def::ScalarDefinition;
pub use schema_def::SchemaDefinition;
pub use selection_set::{Selection, SelectionSet};
pub use string_value::StringValue;
pub use ty::Type_;
pub use union_def::UnionDefinition;
pub use value::Value;
pub use variable::VariableDefinition;
